use std::ffi::CStr;

use assert_cmd::assert::{Assert, AssertResult};
use serde::Deserialize;

use crate::Symbols;

use super::SNAPSHOTS;

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(super) struct Snapshots {
    metas_json: String,
}

pub(crate) unsafe fn exec(Symbols { metas, .. }: Symbols<'_>) -> anyhow::Result<()> {
    let metas_json = metas();
    let metas_json = CStr::from_ptr(metas_json).to_str()?;
    std::assert_eq!(SNAPSHOTS.metas.metas_json, super::sha256(metas_json));
    Ok(())
}

pub(crate) fn assert_output(assert: Assert) -> AssertResult {
    assert.try_success()?.try_stdout("")?.try_stderr("")
}
