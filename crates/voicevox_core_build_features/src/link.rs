use std::{
    env::{self, consts::DLL_PREFIX},
    process::{Command, Output},
    sync::LazyLock,
};

use camino::Utf8Path;

static TARGET: LazyLock<String> = LazyLock::new(|| env::var("TARGET").unwrap());
static HOST: LazyLock<String> = LazyLock::new(|| env::var("HOST").unwrap());

pub fn link(attempt_pkg_config: bool) -> anyhow::Result<()> {
    if *HOST == *TARGET && attempt_pkg_config {
        let mut version = include_str!("../onnxruntime-version.txt").split('.');
        let major_ver = version
            .next()
            .and_then(|s| s.parse::<u64>().ok())
            .expect("could not parse");
        let minor_ver = version.next().expect("could not parse");

        match pkg_config::Config::new()
            .range_version(&*format!("{major_ver}.{minor_ver}")..&(major_ver + 1).to_string())
            .probe(&format!("{DLL_PREFIX}onnxruntime"))
        {
            Ok(_) => return Ok(()),
            Err(err) => {
                for line in err.to_string().trim_start().lines() {
                    println!("cargo::warning={line}");
                }
            }
        }
    }

    let rust_target = TARGET.split('-').collect::<Vec<_>>();

    if rust_target.contains(&"windows") {
        println!("cargo::rustc-link-lib=static=onnxruntime"); // import libraryの方
    } else {
        println!("cargo::rustc-link-lib=dylib=onnxruntime");
    }

    if rust_target.contains(&"ios") {
        if let Some(dir) = Command::new("xcrun")
            .args(["clang", "--print-resource-dir"])
            .output()
            .ok()
            .filter(|Output { status, .. }| status.success())
            .and_then(|Output { stdout, .. }| String::from_utf8(stdout).ok())
            .map(|resource_dir| {
                Utf8Path::new(resource_dir.trim())
                    .join("lib")
                    .join("darwin")
            })
        {
            println!("cargo::rustc-link-search=native={dir}");
        }
        if rust_target.contains(&"sim") {
            println!("cargo::rustc-link-lib=clang_rt.iossim");
        } else {
            println!("cargo::rustc-link-lib=clang_rt.ios");
        }
    }
    Ok(())
}
