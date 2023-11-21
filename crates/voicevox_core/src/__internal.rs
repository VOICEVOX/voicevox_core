pub mod doctest_fixtures;

// VOICEVOX CORE内のラッパー向けの実装
// FIXME: 要議論: https://github.com/VOICEVOX/voicevox_core/issues/595

pub fn to_zenkaku(surface: &str) -> String {
    crate::user_dict::to_zenkaku(surface)
}

pub fn validate_pronunciation(pronunciation: &str) -> crate::Result<()> {
    crate::user_dict::validate_pronunciation(pronunciation).map_err(Into::into)
}
