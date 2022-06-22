#[cfg(not(feature = "generate_bindings"))]
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/generated/bindings.rs"
));
