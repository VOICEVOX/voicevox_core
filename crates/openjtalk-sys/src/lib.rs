extern crate link_cplusplus;
#[allow(clippy::all)]
#[allow(warnings)]
mod bindings {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/generated/bindings.rs"
    ));
}
pub use bindings::*;
