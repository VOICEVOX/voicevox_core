use std::{env, path::Path};

use indoc::formatdoc;
use quote::quote;

fn main() -> anyhow::Result<()> {
    let out_dir = &env::var("OUT_DIR").unwrap();

    let version_macro = formatdoc! {"
        #[macro_export]
        macro_rules! version {{
            () => {{
                {version}
            }};
        }}
        ",
        version = quote!(#VERSION),
    };

    fs_err::write(
        Path::new(out_dir).join("decl_version_macro.rs"),
        version_macro,
    )?;
    return Ok(());

    const VERSION: &str = env!("CARGO_PKG_VERSION");
}
