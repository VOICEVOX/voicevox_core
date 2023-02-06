use std::ffi::CStr;

use assert_cmd::assert::Assert;

use crate::Symbols;

pub(crate) unsafe fn exec(
    Symbols {
        voicevox_get_version,
    }: Symbols<'_>,
) -> anyhow::Result<()> {
    let version = voicevox_get_version();
    let version = CStr::from_ptr(version).to_str()?;
    println!("Version: {version:?}");
    Ok(())
}

pub(crate) fn assert(assert: Assert) {
    assert.success().stdout("Version: \"0.0.0\"\n").stderr("");
}
