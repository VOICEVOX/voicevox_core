extern crate link_cplusplus;
#[allow(clippy::all)]
#[allow(warnings)]
#[cfg(not(feature = "generate-bindings"))]
mod bindings {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/generated/bindings.rs"
    ));
}
#[cfg(not(feature = "generate-bindings"))]
pub use bindings::*;
