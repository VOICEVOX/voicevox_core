#[macro_export]
macro_rules! unwrap_with_throw {
    ($env:expr, $expr:expr) => {
        match $expr {
            Ok(value) => value,
            Err(error) => {
                $env.throw_new("jp/Hiroshiba/VoicevoxCore/VoicevoxError", error.to_string())
                    .unwrap();
                return 0;
            }
        }
    };
}
