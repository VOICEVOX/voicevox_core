/// # Panics
///
/// 出力結果が次の場合にパニックする。
///
/// - アクセント位置として`0`が存在する。
/// - 母音部分に母音以外の音素が置かれている。
pub(super) trait FullcontextExtractor {
    fn extract_fullcontext(&self, text: &str) -> anyhow::Result<Vec<String>>;
}

#[cfg(target_os = "emscripten")]
include!("open_jtalk.wasm.rs");

#[cfg(not(target_os = "emscripten"))]
include!("open_jtalk.native.rs");
