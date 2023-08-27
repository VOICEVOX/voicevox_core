use std::fmt::Display;

pub fn to_zenkaku(surface: &str) -> String {
    crate::user_dict::to_zenkaku(surface)
}

pub fn validate_pronunciation(pronunciation: &str) -> std::result::Result<(), impl Display> {
    crate::user_dict::validate_pronunciation(pronunciation)
}
