use std::{
    collections::HashMap,
    convert, env,
    io::{self, Read},
    path::Path,
    sync::LazyLock,
};

use anyhow::{anyhow, bail, ensure, Context as _};
use camino::Utf8PathBuf;
use itertools::Itertools as _;
use serde::{de::Error as _, Deserialize, Deserializer};
use serde_with::{serde_as, DeserializeAs};
use sha2::{Digest as _, Sha256};

#[cfg(not(any(feature = "load-onnxruntime", feature = "link-onnxruntime")))]
compile_error!("either `load-onnxruntime` or `link-onnxruntime` must be enabled");

const ENV_FORCE_DOWNLOAD_ORT: &str = "VVCORE_BUILD_FORCE_DOWNLOAD_ORT";
const ENV_FORCE_COPY_DOWNLOADED_ORT: &str = "VVCORE_BUILD_FORCE_COPY_DOWNLOADED_ORT";

static FORCE_DOWNLOAD_ORT: LazyLock<bool> = LazyLock::new(|| is_true(ENV_FORCE_DOWNLOAD_ORT));
static FORCE_COPY_DOWNLOADED_ORT: LazyLock<bool> =
    LazyLock::new(|| is_true(ENV_FORCE_COPY_DOWNLOADED_ORT));

static TARGET: LazyLock<String> = LazyLock::new(|| env::var("TARGET").unwrap());
static OUT_DIR: LazyLock<Utf8PathBuf> =
    LazyLock::new(|| env::var("OUT_DIR").expect("`$OUT_DIR` is not UTF-8").into());

#[cfg(feature = "link-onnxruntime")]
static HOST: LazyLock<String> = LazyLock::new(|| env::var("HOST").unwrap());

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    if env::var("DOCS_RS").is_ok() {
        return Ok(());
    }

    println!("cargo::rerun-if-env-changed={ENV_FORCE_DOWNLOAD_ORT}");
    println!("cargo::rerun-if-env-changed={ENV_FORCE_COPY_DOWNLOADED_ORT}");

    println!("cargo::rerun-if-changed=build.rs");

    #[cfg(feature = "link-onnxruntime")]
    {
        use std::{
            env::consts::DLL_PREFIX,
            process::{Command, Output},
        };

        use camino::Utf8Path;

        if *HOST == *TARGET && !*FORCE_DOWNLOAD_ORT {
            let mut version = include_str!("./onnxruntime-version.txt").split('.');
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
            println!("cargo::rustc-link-lib=static=onnxruntime");
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
    }

    if !(cfg!(feature = "link-onnxruntime") || *FORCE_DOWNLOAD_ORT) {
        return Ok(());
    }

    let up_to_date = {
        let current_exe_mtime =
            fs_err::metadata(process_path::get_executable_path().unwrap())?.modified()?;

        move |path: &Path| {
            let metadata = match fs_err::metadata(path) {
                Ok(metadata) => metadata,
                Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(false),
                Err(err) => return Err(err),
            };
            let mtime = metadata.modified()?;
            Ok::<_, io::Error>(mtime <= current_exe_mtime)
        }
    };

    let mut lib_dir = OUT_DIR.ancestors().nth(4).unwrap();
    if lib_dir.file_name() == Some(&TARGET) {
        lib_dir = lib_dir.parent().unwrap();
    }
    let lib_dir = &lib_dir
        .join("voicevox_core")
        .join("downloads")
        .join("onnxruntime");
    fs_err::create_dir_all(lib_dir)?;

    let TargetList {
        repository,
        version,
        targets,
    } = toml::from_str(include_str!("./onnxruntime-libs.toml"))
        .with_context(|| "could not parse onnxruntime-libs.toml")?;

    let Target {
        artifact_name,
        lib_sha256,
        importlib_sha256,
    } = targets
        .get(&*TARGET)
        .with_context(|| format!("`{}` not found in onnxruntime-libs.toml", *TARGET))?;

    let lib_versioned_file_name = &lib_versioned_file_name(&version)?;
    let lib_path = &lib_dir.join(lib_versioned_file_name);

    let importlib_path = &importlib_file_name().map(|s| lib_dir.join(s));

    let lib_symlink_file_name =
        Some(lib_unversioned_file_name()?).filter(|s| s != lib_versioned_file_name);

    // TODO: Rust 1.91なら`std::iter::chain`がある
    if !itertools::chain([lib_path], importlib_path)
        .map(|p| up_to_date(p.as_ref()))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .all(convert::identity)
    {
        let asset_name = format!("{artifact_name}-{version}.tgz");

        let res = reqwest::get(format!(
            "https://github.com/{repository}/releases/download/onnxruntime-{version}/{asset_name}"
        ))
        .await?;
        ensure!(res.status() == 200, "{}", res.status());

        let tgz = &*res.bytes().await?;

        let lib_content = &extract_lib(tgz, lib_versioned_file_name)?;
        verify(lib_content, lib_sha256)?;
        fs_err::write(lib_path, lib_content)?;

        if let Some(importlib_path) = importlib_path {
            let importlib_sha256 =
                &importlib_sha256.with_context(|| "`importlib-sha256` is required for Windows")?;
            let importlib_content = &extract_lib(tgz, "onnxruntime.lib")?;
            verify(importlib_content, importlib_sha256)?;
            fs_err::write(importlib_path, importlib_content)?;
        }
    }

    println!("cargo::rerun-if-changed={lib_path}");
    if let Some(importlib_path) = importlib_path {
        println!("cargo::rerun-if-changed={importlib_path}");
    }

    if cfg!(feature = "link-onnxruntime")
        && cfg!(windows)
        && !lib_dir.join("onnxruntime.lib").try_exists()?
    {
        bail!("missing the import library");
    }

    if let Some(lib_symlink_file_name) = lib_symlink_file_name {
        let dst = &lib_dir.join(lib_symlink_file_name);
        if dst.is_symlink() {
            fs_err::remove_file(dst)?;
        }
        symlink_or_copy(lib_path, dst)?;
        println!("cargo::rerun-if-changed={dst}");
    }

    if cfg!(feature = "link-onnxruntime") {
        println!("cargo::rustc-link-search=native={lib_dir}");
    }

    if cfg!(feature = "link-onnxruntime") || *FORCE_COPY_DOWNLOADED_ORT {
        let dst = OUT_DIR.ancestors().nth(3).unwrap();
        for dst in [dst, &*dst.join("examples"), &*dst.join("deps")] {
            // TODO: Rust 1.91なら`std::iter::chain`がある
            for file_name in itertools::chain([&**lib_versioned_file_name], lib_symlink_file_name) {
                let dst = &dst.join(file_name);
                if !up_to_date(dst.as_ref())? {
                    if dst.is_symlink() {
                        fs_err::remove_file(dst)?;
                    }
                    symlink_or_copy(lib_path, dst)?;
                }
                println!("cargo::rerun-if-changed={dst}");
            }
        }
    }
    Ok(())
}

fn is_true(env_name: &'static str) -> bool {
    env::var(env_name).is_ok_and(|s| ["1", "true"].contains(&&*s.to_ascii_lowercase()))
}

fn lib_versioned_file_name(version: &str) -> anyhow::Result<String> {
    let rust_target = TARGET.split('-').collect::<Vec<_>>();

    if rust_target.contains(&"windows") {
        Ok("onnxruntime.dll".to_owned())
    } else if rust_target.contains(&"apple") {
        Ok(format!("libonnxruntime.{version}.dylib"))
    } else if rust_target.contains(&"unknown") && rust_target.contains(&"linux") {
        Ok(format!("libonnxruntime.so.{version}"))
    } else if rust_target.contains(&"linux") && rust_target.contains(&"android") {
        Ok("libonnxruntime.so".to_owned())
    } else {
        bail!("unknown target tuple: {}", *TARGET);
    }
}

fn lib_unversioned_file_name() -> anyhow::Result<&'static str> {
    let rust_target = TARGET.split('-').collect::<Vec<_>>();

    if rust_target.contains(&"windows") {
        Ok("onnxruntime.dll")
    } else if rust_target.contains(&"apple") {
        Ok("libonnxruntime.dylib")
    } else if rust_target.contains(&"linux") {
        Ok("libonnxruntime.so")
    } else {
        bail!("unknown target tuple: {}", *TARGET);
    }
}

fn importlib_file_name() -> Option<&'static str> {
    let rust_target = TARGET.split('-').collect::<Vec<_>>();
    (rust_target.contains(&"windows")).then_some("onnxruntime.lib")
}

fn verify(content: &[u8], expected_sha256: &[u8; 32]) -> anyhow::Result<()> {
    let actual_sha256 = Sha256::digest(content);
    ensure!(
        actual_sha256[..] == expected_sha256[..],
        "SHA256 mismatch: expected {expected_sha256}, got {actual_sha256}",
        expected_sha256 = hex::encode(expected_sha256),
        actual_sha256 = hex::encode(&actual_sha256[..]),
    );
    Ok(())
}

fn extract_lib(tgz: &[u8], lib_name: &str) -> anyhow::Result<Vec<u8>> {
    return binstall_tar::Archive::new(flate2::read::GzDecoder::new(tgz))
        .entries()?
        .map(|entry| {
            let entry = entry?;
            let path = entry.path()?;
            match *path.components().collect::<Vec<_>>() {
                [_, c1, c2] if c1.as_os_str() == "lib" && c2.as_os_str() == lib_name => {
                    read_bytes(entry).map(Some)
                }
                _ => Ok(None),
            }
        })
        .flat_map(Result::transpose)
        .collect::<io::Result<Vec<_>>>()?
        .into_iter()
        .exactly_one()
        .map_err(|_| anyhow!("could not find 'lib/{lib_name}' in the asset"));

    fn read_bytes<'a>(entry: binstall_tar::Entry<'a, impl Read + 'a>) -> io::Result<Vec<u8>> {
        let mut buf = Vec::with_capacity(entry.size() as _);
        { entry }.read_to_end(&mut buf)?;
        Ok(buf)
    }
}

#[cfg(unix)]
fn symlink_or_copy(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    use relative_path::PathExt as _;

    let src = src.as_ref();
    let dst = dst.as_ref();
    let target = src
        .relative_to(dst.parent().unwrap())
        .map(|target| target.into_string().into())
        .unwrap_or_else(|_| src.to_owned());
    fs_err::os::unix::fs::symlink(target, dst)
}

#[cfg(not(unix))]
fn symlink_or_copy(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs_err::copy(src, dst)?;
    Ok(())
}

#[derive(Deserialize, Debug)]
struct TargetList {
    repository: String,
    version: String,
    targets: HashMap<String, Target>,
}

#[serde_as]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
struct Target {
    artifact_name: String,

    #[serde_as(as = "ThirtyTwoBytes")]
    lib_sha256: [u8; 32],

    #[serde_as(as = "Option<ThirtyTwoBytes>")]
    importlib_sha256: Option<[u8; 32]>,
}

struct ThirtyTwoBytes;

impl<'de> DeserializeAs<'de, [u8; 32]> for ThirtyTwoBytes {
    fn deserialize_as<D>(deserializer: D) -> Result<[u8; 32], D::Error>
    where
        D: Deserializer<'de>,
    {
        hex::decode(String::deserialize(deserializer)?)
            .map_err(D::Error::custom)?
            .try_into()
            .map_err(|_| D::Error::custom("expected 32 bytes"))
    }
}
