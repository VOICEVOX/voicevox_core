use std::{io, path::PathBuf};

use eyre::ensure;

#[derive(clap::Parser)]
pub(crate) struct ArgsGenerateCHeader {
    /// Generate bindings and compare it to the existing bindings file and error if they are different
    #[clap(long)]
    verify: bool,

    /// The file to output the bindings to
    #[clap(short, long)]
    output: Option<PathBuf>,
}

pub(crate) fn run(ArgsGenerateCHeader { verify, output }: ArgsGenerateCHeader) -> eyre::Result<()> {
    let bindings = cbindgen::generate(CRATE_DIR)?;

    if let Some(output) = output {
        let changed = bindings.write_to_file(&output);
        ensure!(
            !(verify && changed),
            "Bindings changed: {}",
            output.display(),
        );
    } else {
        bindings.write(io::stdout());
    }
    return Ok(());

    static CRATE_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../voicevox_core_c_api");
}
