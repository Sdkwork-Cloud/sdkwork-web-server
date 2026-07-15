use std::{
    future::Future,
    io,
    pin::Pin,
    task::{Context, Poll},
};

use axum_server::accept::Accept;
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    sync::{OwnedSemaphorePermit, Semaphore},
};

#[derive(Clone)]
pub struct ConnectionLimitAcceptor<A> {
    inner: A,
    global_permits: std::sync::Arc<Semaphore>,
    listener_permits: std::sync::Arc<Semaphore>,
}

impl<A> ConnectionLimitAcceptor<A> {
    pub fn new(
        inner: A,
        global_permits: std::sync::Arc<Semaphore>,
        maximum_listener_connections: usize,
    ) -> Self {
        Self {
            inner,
            global_permits,
            listener_permits: std::sync::Arc::new(Semaphore::new(maximum_listener_connections)),
        }
    }
}

impl<A, I, S> Accept<I, S> for ConnectionLimitAcceptor<A>
where
    A: Accept<I, S> + Clone + Send + Sync + 'static,
    A::Future: Send + 'static,
    A::Stream: Send + Unpin + 'static,
    A::Service: Send + 'static,
    I: Send + 'static,
    S: Send + 'static,
{
    type Stream = ConnectionLimitedStream<A::Stream>;
    type Service = A::Service;
    type Future =
        Pin<Box<dyn Future<Output = io::Result<(Self::Stream, Self::Service)>> + Send + 'static>>;

    fn accept(&self, stream: I, service: S) -> Self::Future {
        let inner = self.inner.clone();
        let global_permits = self.global_permits.clone();
        let listener_permits = self.listener_permits.clone();
        Box::pin(async move {
            let global_permit = global_permits
                .try_acquire_owned()
                .map_err(|_| connection_limit_error())?;
            let listener_permit = listener_permits
                .try_acquire_owned()
                .map_err(|_| connection_limit_error())?;
            let (stream, service) = inner.accept(stream, service).await?;
            Ok((
                ConnectionLimitedStream {
                    inner: stream,
                    _global_permit: global_permit,
                    _listener_permit: listener_permit,
                },
                service,
            ))
        })
    }
}

pub struct ConnectionLimitedStream<I> {
    inner: I,
    _global_permit: OwnedSemaphorePermit,
    _listener_permit: OwnedSemaphorePermit,
}

impl<I: AsyncRead + Unpin> AsyncRead for ConnectionLimitedStream<I> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        context: &mut Context<'_>,
        buffer: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_read(context, buffer)
    }
}

impl<I: AsyncWrite + Unpin> AsyncWrite for ConnectionLimitedStream<I> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        context: &mut Context<'_>,
        buffer: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        Pin::new(&mut self.inner).poll_write(context, buffer)
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        context: &mut Context<'_>,
    ) -> Poll<Result<(), io::Error>> {
        Pin::new(&mut self.inner).poll_flush(context)
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        context: &mut Context<'_>,
    ) -> Poll<Result<(), io::Error>> {
        Pin::new(&mut self.inner).poll_shutdown(context)
    }
}

fn connection_limit_error() -> io::Error {
    io::Error::new(io::ErrorKind::ConnectionAborted, "connection limit reached")
}
