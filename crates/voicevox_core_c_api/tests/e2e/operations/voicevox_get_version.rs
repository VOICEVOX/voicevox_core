use std::ffi::CStr;

use assert_cmd::assert::{Assert, AssertResult};

use crate::Symbols;

pub(crate) unsafe fn exec(
    Symbols {
        voicevox_get_version,
        ..
    }: Symbols<'_>,
) -> anyhow::Result<()> {
    let version = voicevox_get_version();
    let version = CStr::from_ptr(version).to_str()?;
    std::assert_eq!(env!("CARGO_PKG_VERSION"), version);
    Ok(())
}

pub(crate) fn assert_output(assert: Assert) -> AssertResult {
    assert.try_success()?.try_stdout("")?.try_stderr("")
}
