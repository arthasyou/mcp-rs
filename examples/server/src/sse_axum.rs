use std::{collections::HashMap, sync::Arc};

use axum::{
    Router,
    extract::{Extension, Query},
    response::{Sse, sse::Event},
    routing::get,
};
use futures::{Stream, TryStreamExt};
use mcp_core_rs::{protocol::message::JsonRpcMessage, utils::CleanupStream};
use mcp_server_rs::{
    router::{impls::chart::ChartRouter, service::RouterService},
    server::Server,
};
use mcp_transport_rs::server::sse::SseTransport;
use tokio::{
    io,
    sync::{RwLock, mpsc},
};
use tokio_stream::{StreamExt, once, wrappers::UnboundedReceiverStream};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

type SessionId = Arc<str>;
type SseSender = mpsc::UnboundedSender<JsonRpcMessage>;

#[derive(Clone, Default)]
pub struct App {
    channels: Arc<RwLock<HashMap<SessionId, SseSender>>>,
}

fn session_id() -> SessionId {
    Arc::from(format!("{:016x}", rand::random::<u128>()))
}

#[tokio::main]
async fn main() -> io::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug,sse_axum=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = App::default();
    let router = Router::new()
        .route("/sse", get(sse_handler).post(post_handler))
        .layer(axum::Extension(app));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:18000").await?;
    axum::serve(listener, router).await
}

async fn sse_handler(
    Extension(app): Extension<App>,
) -> Sse<impl Stream<Item = Result<Event, io::Error>>> {
    let session = session_id();
    tracing::info!(%session, "new SSE connection");

    let (to_client_tx, to_client_rx) = mpsc::unbounded_channel::<JsonRpcMessage>();
    let (to_server_tx, to_server_rx) = mpsc::unbounded_channel::<JsonRpcMessage>();
    app.channels
        .write()
        .await
        .insert(session.clone(), to_server_tx.clone());

    // 添加 shutdown 通道
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    let session_for_task = session.clone();
    let channels = app.channels.clone();

    // 启动服务，支持主动清理
    tokio::spawn(async move {
        let transport = SseTransport::new(to_client_tx, to_server_rx);
        let router = RouterService(ChartRouter::new());
        let server = Server::new(router);

        let result = tokio::select! {
            res = server.run(transport) => {
                tracing::info!(%session_for_task, "server.run completed");
                res
            },
            _ = shutdown_rx => {
                tracing::info!(%session_for_task, "client disconnected, cleaning up");
                Ok(())
            }
        };

        tracing::info!(%session_for_task, "Cleaning up session");
        channels.write().await.remove(&session_for_task);

        if let Err(e) = result {
            tracing::error!(?e, "server run error");
        }
    });

    // 第一条 event: endpoint，带上 session_id
    let init_event = Event::default()
        .event("endpoint")
        .data(format!("?sessionId={}", session));

    let init_stream = once(Ok::<Event, io::Error>(init_event));

    // 后续是 stream: JsonRpcMessage => event: message
    let message_stream = UnboundedReceiverStream::new(to_client_rx)
        .map(|msg| {
            serde_json::to_string(&msg).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
        })
        .and_then(|json| futures::future::ok(Event::default().event("message").data(json)));

    let full_stream = init_stream.chain(message_stream);

    let full_stream = CleanupStream {
        inner: full_stream,
        shutdown_tx: Some(shutdown_tx),
    };

    Sse::new(full_stream)
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostQuery {
    pub session_id: String,
}

async fn post_handler(
    Extension(app): Extension<App>,
    Query(PostQuery { session_id }): Query<PostQuery>,
    body: axum::body::Bytes,
) -> Result<String, String> {
    let channels = app.channels.read().await;
    if let Some(sender) = channels.get(session_id.as_str()) {
        let data = String::from_utf8(body.to_vec()).map_err(|e| e.to_string())?;
        let msg: JsonRpcMessage = serde_json::from_str(&data).map_err(|e| e.to_string())?;
        println!("Received message: {:?}", &msg);
        sender
            .send(msg)
            .map_err(|_| "Failed to send message".to_string())?;

        // In a real case, this message would be pushed to a proper transport reader.
        // But here we just log it.
        // tracing::info!(?msg, "Received POST message");

        Ok("Accepted".to_string())
    } else {
        Err("Session not found".to_string())
    }
}
