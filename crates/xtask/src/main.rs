mod commands;

use clap::Parser as _;

use crate::commands::update_c_header::ArgsUpdateCHeader;

#[derive(clap::Parser)]
enum Args {
    /// Update voicevox_core.h
    UpdateCHeader(ArgsUpdateCHeader),
}

fn main() -> eyre::Result<()> {
    let args = Args::parse();
    color_eyre::install()?;
    match args {
        Args::UpdateCHeader(args) => commands::update_c_header::run(args),
    }
}
