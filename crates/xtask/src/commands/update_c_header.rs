use std::str;

use eyre::ensure;

#[derive(clap::Parser)]
pub(crate) struct ArgsUpdateCHeader {
    /// Generate bindings and compare it to the existing bindings file and error if they are different
    #[clap(long)]
    verify: bool,
}

pub(crate) fn run(ArgsUpdateCHeader { verify }: ArgsUpdateCHeader) -> eyre::Result<()> {
    let bindings = cbindgen::generate(CRATE_DIR)?;
    let changed = bindings.write_to_file(OUTPUT);
    ensure!(!(verify && changed), "Bindings changed: {OUTPUT}");
    return Ok(());

    static CRATE_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../voicevox_core_c_api");
    static OUTPUT: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../voicevox_core_c_api/include/voicevox_core.h",
    );
}
