fn main() {
    #[cfg(target_os = "linux")]
    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN");

    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path/");
        println!("cargo:rustc-link-arg=-Wl,-install_name,@rpath/libvoicevox_core.dylib");
    }
}
