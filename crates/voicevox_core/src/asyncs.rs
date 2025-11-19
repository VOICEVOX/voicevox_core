//! 非同期操作の実装の切り替えを行う。
//!
//! 「[ブロッキング版API]」と「[非同期版API]」との違いはここに集約される…予定。現在は[`crate::voice_model`]のみで利用している。
//!
//! # Motivation
//!
//! [blocking]クレートで駆動する非同期処理はランタイムが無くても動作する。そのため非同期版APIをもとにブロッキング版APIを構成することはできる。しかし将来WASMビルドすることを考えると、スレッドがまともに扱えないため機能しなくなってしまう。そのためWASM化を見越したブロッキング版APIのために[`SingleTasked`]を用意している。
//!
//! [ブロッキング版API]: crate::blocking
//! [非同期版API]: crate::nonblocking

use std::{
    io::{self, Read as _, Seek as _, SeekFrom},
    ops::DerefMut,
    path::Path,
    pin::Pin,
    task::{self, Poll},
};

use blocking::Unblock;
use futures_io::{AsyncRead, AsyncSeek};
use futures_util::ready;

pub(crate) trait Async: 'static {
    type Mutex<T: Send + Sync + Unpin>: Mutex<T>;
    type RoFile: AsyncRead + AsyncSeek + Send + Sync + Unpin;

    /// ファイルを読み取り専用(RO)で開く。
    ///
    /// `io::Error`は素（`i32`相当）のままにしておき、この関数を呼び出す側でfs-err風のメッセージを付
    /// ける。
    async fn open_file_ro(path: impl AsRef<Path>) -> io::Result<Self::RoFile>;

    async fn read(path: impl AsRef<Path>) -> io::Result<Vec<u8>>;

    async fn write(path: impl AsRef<Path>, content: impl AsRef<[u8]>) -> io::Result<()>;
}

pub(crate) trait Mutex<T>: From<T> + Send + Sync + Unpin {
    async fn lock(&self) -> impl DerefMut<Target = T>;
}

/// エグゼキュータが非同期タスクの並行実行をしないことを仮定する、[`Async`]の実装。
///
/// [ブロッキング版API]用。
///
/// # Performance
///
/// `async`の中でブロッキング操作を直接行う。そのためTokioやasync-stdのような通常の非同期ランタイム
/// 上で動くべきではない。
///
/// [ブロッキング版API]: crate::blocking
pub(crate) enum SingleTasked {}

impl Async for SingleTasked {
    type Mutex<T: Send + Sync + Unpin> = StdMutex<T>;
    type RoFile = StdFile;

    async fn open_file_ro(path: impl AsRef<Path>) -> io::Result<Self::RoFile> {
        std::fs::File::open(path).map(StdFile)
    }

    async fn read(path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
        std::fs::read(path)
    }

    async fn write(path: impl AsRef<Path>, content: impl AsRef<[u8]>) -> io::Result<()> {
        std::fs::write(path, content)
    }
}

#[derive(derive_more::Debug)]
#[debug("{_0:?}")]
pub(crate) struct StdMutex<T>(std::sync::Mutex<T>);

impl<T> From<T> for StdMutex<T> {
    fn from(inner: T) -> Self {
        Self(inner.into())
    }
}

impl<T: Send + Sync + Unpin> Mutex<T> for StdMutex<T> {
    async fn lock(&self) -> impl DerefMut<Target = T> {
        self.0.lock().unwrap_or_else(|e| panic!("{e}"))
    }
}

#[derive(derive_more::Debug)]
#[debug("{_0:?}")]
pub(crate) struct StdFile(std::fs::File);

impl AsyncRead for StdFile {
    fn poll_read(
        mut self: Pin<&mut Self>,
        _: &mut task::Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        Poll::Ready(self.0.read(buf))
    }
}

impl AsyncSeek for StdFile {
    fn poll_seek(
        mut self: Pin<&mut Self>,
        _: &mut task::Context<'_>,
        pos: SeekFrom,
    ) -> Poll<io::Result<u64>> {
        Poll::Ready(self.0.seek(pos))
    }
}

/// [blocking]クレートで駆動する[`Async`]の実装。
///
/// [非同期版API]用。
///
/// [非同期版API]: crate::nonblocking
pub(crate) enum BlockingThreadPool {}

impl Async for BlockingThreadPool {
    type Mutex<T: Send + Sync + Unpin> = async_lock::Mutex<T>;
    type RoFile = AsyncRoFile;

    async fn open_file_ro(path: impl AsRef<Path>) -> io::Result<Self::RoFile> {
        AsyncRoFile::open(path).await
    }

    async fn read(path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
        async_fs::read(path).await
    }

    async fn write(path: impl AsRef<Path>, content: impl AsRef<[u8]>) -> io::Result<()> {
        async_fs::write(path, content).await
    }
}

impl<T: Send + Sync + Unpin> Mutex<T> for async_lock::Mutex<T> {
    async fn lock(&self) -> impl DerefMut<Target = T> {
        self.lock().await
    }
}

// TODO: `async_fs::File::into_std_file`みたいなのがあればこんなの↓は作らなくていいはず。PR出す？
#[derive(Debug)]
pub(crate) struct AsyncRoFile {
    // `poll_read`と`poll_seek`しかしない
    unblock: Unblock<std::fs::File>,

    // async-fsの実装がやっているように「正しい」シーク位置を保持する。ただしファイルはパイプではな
    // いことがわかっているため smol-rs/async-fs#4 は考えない
    real_seek_pos: Option<u64>,
}

impl AsyncRoFile {
    async fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref().to_owned();
        let unblock = Unblock::new(blocking::unblock(|| std::fs::File::open(path)).await?);
        Ok(Self {
            unblock,
            real_seek_pos: None,
        })
    }

    pub(crate) async fn close(self) {
        let file = self.unblock.into_inner().await;
        blocking::unblock(|| drop(file)).await;
    }
}

impl AsyncRead for AsyncRoFile {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        if self.real_seek_pos.is_none() {
            self.real_seek_pos = Some(ready!(
                Pin::new(&mut self.unblock).poll_seek(cx, SeekFrom::Current(0))
            )?);
        }
        let n = ready!(Pin::new(&mut self.unblock).poll_read(cx, buf))?;
        *self.real_seek_pos.as_mut().expect("should be present") += n as u64;
        Poll::Ready(Ok(n))
    }
}

impl AsyncSeek for AsyncRoFile {
    fn poll_seek(
        mut self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        pos: SeekFrom,
    ) -> Poll<io::Result<u64>> {
        // async-fsの実装がやっているような"reposition"を行う。
        // https://github.com/smol-rs/async-fs/issues/2#issuecomment-675595170
        if let Some(real_seek_pos) = self.real_seek_pos {
            ready!(Pin::new(&mut self.unblock).poll_seek(cx, SeekFrom::Start(real_seek_pos)))?;
        }
        self.real_seek_pos = None;

        Pin::new(&mut self.unblock).poll_seek(cx, pos)
    }
}
