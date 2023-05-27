use std::{env, path::Path};

use quote::quote;

fn main() -> anyhow::Result<()> {
    let out_dir = &env::var("OUT_DIR").unwrap();

    fs_err::write(Path::new(out_dir).join("version_macro.rs"), version_macro())?;
    Ok(())
}

fn version_macro() -> String {
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
}
