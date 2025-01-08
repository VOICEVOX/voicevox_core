use std::env;

use derive_syn_parse::Parse;
use quote::ToTokens as _;
use serde::Deserialize;
use syn::LitStr;

pub(crate) fn pyproject_project_version(_: Input) -> syn::Result<proc_macro2::TokenStream> {
    let span = proc_macro2::Span::call_site();

    let path = &env::var("CARGO_MANIFEST_DIR")
        .map_err(|e| syn::Error::new(span, format!("could not get `$CARGO_MANIFEST_DIR`: {e}")))?;
    let path = std::path::Path::new(path).join("pyproject.toml");

    let PyprojectToml {
        project: Project { version },
    } = &fs_err::read_to_string(path)
        .map_err(|e| e.to_string())
        .and_then(|s| toml::from_str(&s).map_err(|e| e.to_string()))
        .map_err(|e| syn::Error::new(span, e))?;

    return Ok(LitStr::new(version, span).to_token_stream());

    #[derive(Deserialize)]
    struct PyprojectToml {
        project: Project,
    }

    #[derive(Deserialize)]
    struct Project {
        version: String,
    }
}

#[derive(Parse)]
pub(crate) struct Input {}
