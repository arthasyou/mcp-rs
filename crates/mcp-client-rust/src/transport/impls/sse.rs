use std::{sync::Arc, time::Duration};

use eventsource_client::{Client as SseClient, SSE};
use futures::TryStreamExt;
use mcp_core::protocol::message::{JsonRpcMessage, JsonRpcNotification};
use serde_json;
use service_utils_rs::utils::Request;
use tokio::{
    spawn,
    sync::{Notify, RwLock, mpsc},
    task::JoinHandle,
    time::timeout,
};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, instrument, warn};
use url::Url;

use crate::{
    error::{Error, Result},
    transport::types::{ConnectionState, MessageReceiver, MessageSender},
};

/// SSE Transport 配置
#[derive(Debug, Clone)]
pub struct SseConfig {
    /// 初始重试间隔
    pub initial_retry_interval: Duration,
    /// 最大重试间隔
    pub max_retry_interval: Duration,
    /// 连接超时时间
    pub connection_timeout: Duration,
    /// 是否启用指数退避
    pub exponential_backoff: bool,
    /// 最大重试次数（None 表示无限重试）
    pub max_retries: Option<usize>,
    /// 关闭超时时间
    pub shutdown_timeout: Duration,
}

impl Default for SseConfig {
    fn default() -> Self {
        Self {
            initial_retry_interval: Duration::from_secs(1),
            max_retry_interval: Duration::from_secs(30),
            connection_timeout: Duration::from_secs(10),
            exponential_backoff: true,
            max_retries: None,
            shutdown_timeout: Duration::from_secs(5),
        }
    }
}

/// SSE Transport
pub struct SseTransport {
    pub url: String,
    pub post_endpoint: Arc<RwLock<Option<String>>>,
    pub http_client: Request,
    pub shutdown: CancellationToken,
    pub config: SseConfig,
    pub state: Arc<RwLock<ConnectionState>>,
    pub background_task: Option<JoinHandle<()>>,
    pub endpoint_ready: Arc<Notify>,
    pub message_sender: MessageSender,
    pub message_receiver: Option<MessageReceiver>,
    pub retry_count: Arc<RwLock<usize>>,
}

impl SseTransport {
    /// 创建新的 SSE Transport 数据
    pub fn new(url: impl Into<String>) -> Self {
        Self::with_config(url, SseConfig::default())
    }

    /// 使用配置创建 SSE Transport 数据
    pub fn with_config(url: impl Into<String>, config: SseConfig) -> Self {
        let mut http_client = Request::new();
        http_client
            .set_default_headers(vec![("Content-Type", "application/json".to_string())])
            .unwrap();

        let (tx, rx) = mpsc::unbounded_channel();

        Self {
            url: url.into(),
            post_endpoint: Arc::new(RwLock::new(None)),
            http_client,
            shutdown: CancellationToken::new(),
            config,
            state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            background_task: None,
            endpoint_ready: Arc::new(Notify::new()),
            message_sender: tx,
            message_receiver: Some(rx),
            retry_count: Arc::new(RwLock::new(0)),
        }
    }

    /// 获取当前连接状态
    pub async fn get_state(&self) -> ConnectionState {
        self.state.read().await.clone()
    }

    /// 设置连接状态
    async fn set_state(&self, new_state: ConnectionState) {
        let mut state = self.state.write().await;
        debug!(
            "Connection state transition: {:?} -> {:?}",
            *state, new_state
        );
        *state = new_state;
    }

    /// 获取 POST endpoint
    async fn get_post_endpoint(&self) -> Result<String> {
        let guard = self.post_endpoint.read().await;
        match &*guard {
            Some(endpoint) => Ok(endpoint.clone()),
            None => Err(Error::System(
                "POST endpoint not discovered yet".to_string(),
            )),
        }
    }

    /// 等待 endpoint 就绪
    #[instrument(skip(self))]
    async fn wait_for_endpoint(&self) -> Result<()> {
        let wait_future = self.endpoint_ready.notified();

        match timeout(self.config.connection_timeout, wait_future).await {
            Ok(_) => {
                debug!("Endpoint ready");
                Ok(())
            }
            Err(_) => {
                error!("Timeout waiting for POST endpoint discovery");
                Err(Error::System(
                    "Timeout waiting for POST endpoint discovery".to_string(),
                ))
            }
        }
    }

    /// 建立连接
    #[instrument(skip(self))]
    pub async fn connect(&mut self) -> Result<()> {
        // 检查当前状态
        let current_state = self.get_state().await;
        if current_state != ConnectionState::Disconnected
            && current_state != ConnectionState::Closed
        {
            return Err(Error::System(format!(
                "Cannot start: transport is in {:?} state",
                current_state
            )));
        }

        self.set_state(ConnectionState::Connecting).await;

        let sse_url = self.url.clone();
        let post_endpoint = self.post_endpoint.clone();
        let shutdown = self.shutdown.clone();
        let message_sender = self.message_sender.clone();
        let endpoint_ready = self.endpoint_ready.clone();
        let state = self.state.clone();
        let config = self.config.clone();
        let retry_count = self.retry_count.clone();

        let handle = spawn(async move {
            handle_messages_loop(
                sse_url,
                post_endpoint,
                message_sender,
                endpoint_ready,
                state,
                config,
                retry_count,
                shutdown,
            )
            .await;
        });

        self.background_task = Some(handle);

        // 等待 endpoint 就绪
        self.wait_for_endpoint().await?;
        self.set_state(ConnectionState::Connected).await;

        Ok(())
    }

    /// 断开连接
    #[instrument(skip(self))]
    pub async fn disconnect(&mut self) -> Result<()> {
        let current_state = self.get_state().await;
        if current_state == ConnectionState::Closing || current_state == ConnectionState::Closed {
            return Ok(());
        }

        info!("Closing SSE transport");
        self.set_state(ConnectionState::Closing).await;

        // 发送关闭信号
        self.shutdown.cancel();

        // 等待后台任务完成
        if let Some(handle) = self.background_task.take() {
            match timeout(self.config.shutdown_timeout, handle).await {
                Ok(Ok(_)) => debug!("Background task completed successfully"),
                Ok(Err(e)) => warn!("Background task panicked: {}", e),
                Err(_) => warn!("Shutdown timeout exceeded"),
            }
        }

        self.set_state(ConnectionState::Closed).await;
        info!("SSE transport closed");

        Ok(())
    }

    /// 发送通知（不等待响应）
    #[instrument(skip(self, notification))]
    pub async fn send_message(&self, notification: JsonRpcNotification) -> Result<()> {
        // 检查连接状态
        let state = self.get_state().await;
        if state != ConnectionState::Connected {
            return Err(Error::System(format!(
                "Cannot send notification: transport is in {:?} state",
                state
            )));
        }

        let post_url = self.get_post_endpoint().await?;
        let message = JsonRpcMessage::Notification(notification);

        self.http_client
            .post(&post_url, &serde_json::to_value(&message)?, None)
            .await
            .map_err(|e| {
                warn!("Failed to send notification: {}", e);
                Error::System(e.to_string())
            })?;

        Ok(())
    }

    /// 获取消息接收器
    pub fn take_message_receiver(&mut self) -> Option<MessageReceiver> {
        self.message_receiver.take()
    }
}

// 后台消息处理循环
async fn handle_messages_loop(
    sse_url: String,
    post_endpoint: Arc<RwLock<Option<String>>>,
    message_sender: MessageSender,
    endpoint_ready: Arc<Notify>,
    state: Arc<RwLock<ConnectionState>>,
    config: SseConfig,
    retry_count: Arc<RwLock<usize>>,
    shutdown: CancellationToken,
) {
    let mut retries = 0;

    loop {
        if shutdown.is_cancelled() {
            info!("Shutdown signal received, stopping message loop");
            break;
        }

        // 检查最大重试次数
        if let Some(max_retries) = config.max_retries {
            if retries >= max_retries {
                error!("Maximum retry count ({}) exceeded", max_retries);
                break;
            }
        }

        let result = handle_messages_once(
            sse_url.clone(),
            post_endpoint.clone(),
            message_sender.clone(),
            endpoint_ready.clone(),
            shutdown.clone(),
        )
        .await;

        if shutdown.is_cancelled() {
            break;
        }

        match result {
            Ok(_) => {
                warn!("SSE handler exited normally, retrying...");
                *retry_count.write().await = 0;
            }
            Err(e) => {
                retries = *retry_count.write().await;
                retries += 1;
                *retry_count.write().await = retries;

                let delay = calculate_retry_delay(&config, retries);
                warn!(
                    "SSE connection failed: {}, retry {} in {:?}...",
                    e, retries, delay
                );

                tokio::time::sleep(delay).await;
            }
        }
    }

    *state.write().await = ConnectionState::Closed;
}

fn calculate_retry_delay(config: &SseConfig, retry_count: usize) -> Duration {
    if !config.exponential_backoff {
        return config.initial_retry_interval;
    }

    let delay = config.initial_retry_interval * 2_u32.pow(retry_count.min(10) as u32);
    delay.min(config.max_retry_interval)
}

#[instrument(skip_all)]
async fn handle_messages_once(
    sse_url: String,
    post_endpoint: Arc<RwLock<Option<String>>>,
    message_sender: MessageSender,
    endpoint_ready: Arc<Notify>,
    shutdown: CancellationToken,
) -> Result<()> {
    debug!("Establishing SSE connection to {}", sse_url);

    let client = eventsource_client::ClientBuilder::for_url(&sse_url)
        .map_err(|e| {
            error!("Failed to build SSE client: {}", e);
            Error::System(format!("Failed to build SSE client: {}", e))
        })?
        .build();

    let mut stream = client.stream();

    // 等待 endpoint 事件
    let endpoint_discovered = wait_for_endpoint_event(
        &mut stream,
        &sse_url,
        post_endpoint.clone(),
        endpoint_ready.clone(),
    )
    .await?;

    if !endpoint_discovered {
        return Err(Error::System(
            "Failed to discover POST endpoint".to_string(),
        ));
    }

    info!("SSE connection established, processing messages");

    // 处理消息流
    process_message_stream(stream, message_sender, shutdown).await?;

    Ok(())
}

async fn wait_for_endpoint_event(
    stream: &mut (impl TryStreamExt<Ok = SSE, Error = eventsource_client::Error> + Unpin),
    sse_url: &str,
    post_endpoint: Arc<RwLock<Option<String>>>,
    endpoint_ready: Arc<Notify>,
) -> Result<bool> {
    while let Ok(Some(event)) = stream.try_next().await {
        if let SSE::Event(e) = event {
            if e.event_type == "endpoint" {
                let base_url = Url::parse(sse_url)
                    .map_err(|e| Error::System(format!("Invalid base URL: {}", e)))?;
                let post_url = base_url
                    .join(&e.data)
                    .map_err(|e| Error::System(format!("Failed to resolve endpoint URL: {}", e)))?;

                info!("Discovered SSE POST endpoint: {}", post_url);
                *post_endpoint.write().await = Some(post_url.to_string());
                endpoint_ready.notify_waiters();
                return Ok(true);
            }
        }
    }

    Ok(false)
}

async fn process_message_stream(
    mut stream: impl TryStreamExt<Ok = SSE, Error = eventsource_client::Error> + Unpin,
    message_sender: MessageSender,
    shutdown: CancellationToken,
) -> Result<()> {
    loop {
        tokio::select! {
            maybe_event = stream.try_next() => {
                match maybe_event {
                    Ok(Some(SSE::Event(e))) if e.event_type == "message" => {
                        process_sse_message(
                            e.data,
                            &message_sender,
                        ).await;
                    }
                    Ok(Some(_)) => continue,
                    Ok(None) => {
                        warn!("SSE stream ended");
                        break;
                    }
                    Err(e) => {
                        error!("SSE stream error: {}", e);
                        break;
                    }
                }
            },
            _ = shutdown.cancelled() => {
                info!("Shutdown signal received in message processor");
                break;
            }
        }
    }

    Ok(())
}

async fn process_sse_message(data: String, message_sender: &MessageSender) {
    match serde_json::from_str::<JsonRpcMessage>(&data) {
        Ok(message) => {
            if let Err(e) = message_sender.send(message) {
                warn!("Failed to send message through channel: {}", e);
            }
        }
        Err(err) => {
            warn!("Failed to parse SSE message: {}", err);
        }
    }
}
