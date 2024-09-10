//! 非同期操作の実装の切り替えを行う。
//!
//! 「[ブロッキング版API]」と「[非同期版API]」との違いはここに集約される
//! …予定。現在は[`crate::voice_model`]のみで利用している。
//!
//! # Motivation
//!
//! [blocking]クレートで駆動する非同期処理はランタイムが無くても動作する。そのため非同期版APIを
//! もとにブロッキング版APIを構成することはできる。しかし将来WASMビルドすることを考えると、スレッド
//! がまともに扱えないため機能しなくなってしまう。そのためWASM化を見越したブロッキング版APIのため
//! に[`Unstoppable`]を用意している。
//!
//! [ブロッキング版API]: crate::blocking
//! [非同期版API]: crate::tokio
//! [blocking]: https://docs.rs/crate/blocking

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
///
/// [ブロッキング版API]用。
///
/// [ブロッキング版API]: crate::blocking
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
/// [非同期版API]用。
///
/// [blocking]: https://docs.rs/crate/blocking
/// [非同期版API]: crate::tokio
pub(crate) enum BlockingThreadPool {}

impl Async for BlockingThreadPool {
    async fn open_file(path: impl AsRef<Path>) -> io::Result<impl AsyncRead + AsyncSeek + Unpin> {
        async_fs::File::open(path).await
    }
}
