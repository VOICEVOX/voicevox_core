use std::{
    io::{self, Read as _, Seek as _, SeekFrom},
    path::Path,
    pin::Pin,
    task::{self, Poll},
};

use futures_io::{AsyncRead, AsyncSeek};

pub(crate) trait Async: 'static {
    async fn open_file(path: impl AsRef<Path>) -> io::Result<impl AsyncRead + AsyncSeek + Unpin>;
}

/// "async"としての責務を放棄し、すべてをブロックする。
pub(crate) enum Unstoppable {}

impl Async for Unstoppable {
    async fn open_file(path: impl AsRef<Path>) -> io::Result<impl AsyncRead + AsyncSeek + Unpin> {
        return std::fs::File::open(path).map(UnstoppableFile);

        struct UnstoppableFile(std::fs::File);

        impl AsyncRead for UnstoppableFile {
            fn poll_read(
                mut self: Pin<&mut Self>,
                _: &mut task::Context<'_>,
                buf: &mut [u8],
            ) -> Poll<io::Result<usize>> {
                Poll::Ready(self.0.read(buf))
            }
        }

        impl AsyncSeek for UnstoppableFile {
            fn poll_seek(
                mut self: Pin<&mut Self>,
                _: &mut task::Context<'_>,
                pos: SeekFrom,
            ) -> Poll<io::Result<u64>> {
                Poll::Ready(self.0.seek(pos))
            }
        }
    }
}

/// [blocking]クレートで駆動する。
///
/// [blocking](https://docs.rs/crate/blocking)
pub(crate) enum BlockingThreadPool {}

impl Async for BlockingThreadPool {
    async fn open_file(path: impl AsRef<Path>) -> io::Result<impl AsyncRead + AsyncSeek + Unpin> {
        async_fs::File::open(path).await
    }
}
