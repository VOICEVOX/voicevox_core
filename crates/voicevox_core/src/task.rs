// TODO: `Async::unblock`として取り回す

/// ブロッキング操作を非同期化する。
pub(crate) async fn asyncify<F: FnOnce() -> R + Send + 'static, R: Send + 'static>(f: F) -> R {
    blocking::unblock(f).await
}
