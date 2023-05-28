use std::{env, path::PathBuf};

use once_cell::sync::Lazy;
use quote::quote;

fn main() -> anyhow::Result<()> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    fs_err::write(out_dir.join("version_macro.rs"), &*VERSION_MACRO)?;
    Ok(())
}

static VERSION_MACRO: Lazy<String> = Lazy::new(|| {
    return quote! {
        #[macro_export]
        macro_rules! version {
            () => {
                #VERSION
            };
        }
    }
    .to_string();

    const VERSION: &str = env!("CARGO_PKG_VERSION");
});
