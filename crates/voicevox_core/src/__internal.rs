pub mod doctest_fixtures;
pub mod interop;

use crate::engine::user_dict;

// VOICEVOX CORE内のラッパー向けの実装
// FIXME: 要議論: https://github.com/VOICEVOX/voicevox_core/issues/595

pub fn to_zenkaku(surface: &str) -> String {
    user_dict::to_zenkaku(surface)
}

pub fn validate_pronunciation(pronunciation: &str) -> crate::Result<()> {
    user_dict::validate_pronunciation(pronunciation).map_err(Into::into)
}
