use std::str;

use serde::{Deserialize, Deserializer, de::DeserializeOwned};

/// snapshots.tomlの`[$section_name]`部分を`deserialize`する。
macro_rules! section {
    ($section_name:ident $(,)?) => {{
        #[derive(::serde::Deserialize)]
        struct Snapshots<T> {
            $section_name: T,
        }

        ::std::sync::LazyLock::new(|| {
            let Snapshots { $section_name } =
                ::toml::from_str(crate::snapshots::SNAPSHOTS_TOML).unwrap();
            $section_name
        })
    }};
}
pub(crate) use section;

pub(crate) static SNAPSHOTS_TOML: &str = include_str!("./snapshots.toml");

pub(crate) fn deserialize_platform_specific_snapshot<'de, T, D>(
    deserializer: D,
) -> Result<T, D::Error>
where
    T: DeserializeOwned,
    D: Deserializer<'de>,
{
    let PlatformSpecificSnapshot { __value } = PlatformSpecificSnapshot::deserialize(deserializer)?;
    return Ok(__value);

    #[derive(Deserialize)]
    struct PlatformSpecificSnapshot<T> {
        #[cfg_attr(windows, serde(rename = "windows"))]
        #[cfg_attr(unix, serde(rename = "unix"))]
        __value: T,
    }
}
