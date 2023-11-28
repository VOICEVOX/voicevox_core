use std::panic;

/// ブロッキング操作を非同期化する。
///
/// # Panics
///
/// - `f`がパニックした場合、パニックがそのままunwindされる。
/// - tokioのランタイムの都合で`f`の実行が"cancel"された場合パニックする。
pub(crate) async fn asyncify<F: FnOnce() -> R + Send + 'static, R: Send + 'static>(f: F) -> R {
    tokio::task::spawn_blocking(f)
        .await
        .unwrap_or_else(|err| match err.try_into_panic() {
            Ok(panic) => panic::resume_unwind(panic),
            Err(err) => panic!("{err}"), // FIXME: エラーとして回収する
        })
}
