// TODO: #802 の時点でiOS以外不要になっているはずなので、このbuild.rsは丸ごと消す
// (iOSのためにbuild_util/make_ios_xcframework.bashの修正は必要)
fn main() {
    #[cfg(target_os = "linux")]
    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN");

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    {
        println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path/");
        println!("cargo:rustc-link-arg=-Wl,-install_name,@rpath/libvoicevox_core.dylib");
    }
}
