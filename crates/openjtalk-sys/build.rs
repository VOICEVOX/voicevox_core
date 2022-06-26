fn main() {
    let dst_dir = cmake::build("openjtalk");
    let lib_dir = dst_dir.join("lib");
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=static=OpenjtalkSys");
    generate_bindings();
}

#[cfg(not(feature = "generate-bindings"))]
fn generate_bindings() {}

#[cfg(feature = "generate-bindings")]
fn generate_bindings() {
    use std::env;
    use std::path::PathBuf;
    println!("cargo:rerun-if-changed=wrapper.hpp");
    println!("cargo:rerun-if-changed=src/generated/bindings.rs");
    let bindings = bindgen::Builder::default()
        .header("wrapper.hpp")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .size_t_is_usize(true)
        .rustfmt_bindings(true)
        .rustified_enum("*")
        .generate()
        .expect("Unable to generate bindings");
    let generated_file = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("src")
        .join("generated")
        .join(env::var("CARGO_CFG_TARGET_OS").unwrap())
        .join(env::var("CARGO_CFG_TARGET_ARCH").unwrap())
        .join("bindings.rs");
    println!("cargo:rerun-if-changed={:?}", generated_file);
    bindings
        .write_to_file(&generated_file)
        .expect("Couldn't write bindings!");
}
