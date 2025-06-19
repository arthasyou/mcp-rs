use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::Stream;
use mcp_core::protocol::JsonRpcMessage;
use mcp_error::{Error, Result};
use pin_project::pin_project;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};

const JSONRPC_VERSION_FIELD: &str = "jsonrpc";
const JSONRPC_EXPECTED_VERSION: &str = "2.0";

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
            Poll::Ready(Ok(0)) => Poll::Ready(None),
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

/// Parses a JSON-RPC message from a string, validating structure and version.
fn parse_json_rpc_message(line: &str) -> Result<JsonRpcMessage> {
    let value: serde_json::Value = serde_json::from_str(line)?;
    if !value.is_object() {
        return Err(Error::InvalidMessage(
            "Message must be a JSON object".into(),
        ));
    }
    let obj = value.as_object().unwrap();

    match obj.get(JSONRPC_VERSION_FIELD) {
        Some(serde_json::Value::String(v)) if v == JSONRPC_EXPECTED_VERSION => {}
        _ => {
            return Err(Error::InvalidMessage(
                "Missing or invalid jsonrpc version".into(),
            ));
        }
    }

    let msg = serde_json::from_value(value)?;
    Ok(msg)
}

impl<R, W> ByteTransport<R, W>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    /// Writes a JSON-RPC message to the underlying writer.
    pub async fn write_message(self: &mut Pin<&mut Self>, msg: JsonRpcMessage) -> Result<()> {
        let json = serde_json::to_string(&msg)?;

        let mut this = self.as_mut().project();
        this.writer.write_all(json.as_bytes()).await?;
        this.writer.write_all(b"\n").await?;
        this.writer.flush().await?;

        Ok(())
    }
}
