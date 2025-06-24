use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::Stream as FuturesStream;

pub struct CleanupStream<S> {
    pub inner: S,
    pub shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

impl<S, T, E> FuturesStream for CleanupStream<S>
where
    S: FuturesStream<Item = Result<T, E>> + Unpin,
{
    type Item = Result<T, E>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let poll = Pin::new(&mut self.inner).poll_next(cx);
        if let Poll::Ready(None) = poll {
            if let Some(tx) = self.shutdown_tx.take() {
                let _ = tx.send(());
            }
        }
        poll
    }
}
