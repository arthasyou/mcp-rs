use std::{
    pin::Pin,
    task::{Context, Poll},
};

use async_trait::async_trait;
use futures::{Stream, stream::StreamExt};
use mcp_core_rs::{protocol::message::JsonRpcMessage, utils::parse_json_rpc_message};
use mcp_error_rs::{Error, Result};
use pin_project::pin_project;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};

use crate::server::traits::ServerTransport;

#[pin_project]
/// A transport that reads and writes JSON-RPC messages over byte streams.
pub struct ByteTransport<R, W> {
    #[pin]
    reader: BufReader<R>,
    #[pin]
    writer: W,
    buf: Vec<u8>,
}

impl<R, W> ByteTransport<R, W>
where
    R: AsyncRead,
    W: AsyncWrite,
{
    /// Creates a new `ByteTransport` with the given reader and writer.
    pub fn new(reader: R, writer: W) -> Self {
        Self {
            reader: BufReader::with_capacity(2 * 1024 * 1024, reader),
            writer,
            buf: Vec::with_capacity(2 * 1024 * 1024),
        }
    }
}

impl<R, W> Stream for ByteTransport<R, W>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    type Item = Result<JsonRpcMessage>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        this.buf.clear();

        let mut reader = this.reader.as_mut();
        let mut read_future = Box::pin(reader.read_until(b'\n', this.buf));
        match read_future.as_mut().poll(cx) {
            Poll::Ready(Ok(0)) => {
                tracing::info!("Client closed connection (read 0 bytes)");
                Poll::Ready(None)
            }
            Poll::Ready(Ok(_)) => {
                let line = match String::from_utf8(std::mem::take(this.buf)) {
                    Ok(s) => s,
                    Err(e) => {
                        tracing::warn!(?e, "Invalid UTF-8 line");
                        return Poll::Ready(Some(Err(Error::Utf8(e))));
                    }
                };
                Poll::Ready(Some(parse_json_rpc_message(&line)))
            }
            Poll::Ready(Err(e)) => Poll::Ready(Some(Err(Error::Io(e)))),
            Poll::Pending => Poll::Pending,
        }
    }
}

#[async_trait]
impl<R, W> ServerTransport for ByteTransport<R, W>
where
    R: AsyncRead + Unpin + Send + Sync,
    W: AsyncWrite + Unpin + Send + Sync,
{
    async fn read_message(&mut self) -> Option<Result<JsonRpcMessage>> {
        self.next().await
    }

    async fn write_message(&mut self, msg: JsonRpcMessage) -> Result<()> {
        let mut this = Pin::new(self).project();
        let json = serde_json::to_string(&msg)?;
        this.writer.write_all(json.as_bytes()).await?;
        this.writer.write_all(b"\n").await?;
        this.writer.flush().await?;
        Ok(())
    }
}
