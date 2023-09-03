// FIXME: 要議論: https://github.com/VOICEVOX/voicevox_core/issues/595

use std::fmt::Display;

pub fn to_zenkaku(surface: &str) -> String {
    crate::user_dict::to_zenkaku(surface)
}

pub fn validate_pronunciation(pronunciation: &str) -> std::result::Result<(), impl Display> {
    crate::user_dict::validate_pronunciation(pronunciation)
}
