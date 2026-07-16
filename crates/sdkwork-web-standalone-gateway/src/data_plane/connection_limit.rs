use std::{
    io,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    sync::{OwnedSemaphorePermit, Semaphore},
};

pub struct ConnectionLimiter {
    global_permits: Arc<Semaphore>,
    listener_permits: Arc<Semaphore>,
}

impl ConnectionLimiter {
    pub fn new(global_permits: Arc<Semaphore>, maximum_listener_connections: usize) -> Self {
        Self {
            global_permits,
            listener_permits: Arc::new(Semaphore::new(maximum_listener_connections)),
        }
    }

    pub fn try_admit<I>(&self, stream: I) -> io::Result<ConnectionLimitedStream<I>> {
        let global_permit = self
            .global_permits
            .clone()
            .try_acquire_owned()
            .map_err(|_| connection_limit_error())?;
        let listener_permit = self
            .listener_permits
            .clone()
            .try_acquire_owned()
            .map_err(|_| connection_limit_error())?;
        Ok(ConnectionLimitedStream {
            inner: stream,
            _global_permit: global_permit,
            _listener_permit: listener_permit,
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
