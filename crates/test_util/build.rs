use std::{
    env,
    io::{self, Cursor, Write as _},
    path::Path,
};

use anyhow::{anyhow, ensure};
use camino::{Utf8Path, Utf8PathBuf};
use cargo_metadata::MetadataCommand;
use flate2::read::GzDecoder;
use indoc::formatdoc;
use tar::Archive;
use zip::{ZipWriter, write::FileOptions};

#[path = "src/typing.rs"]
mod typing;

const DIC_DIR_NAME: &str = "open_jtalk_dic_utf_8-1.11";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let out_dir = &Utf8PathBuf::from(env::var("OUT_DIR").unwrap());
    let dist = &Utf8Path::new(env!("CARGO_MANIFEST_DIR")).join("data");

    let dic_dir = dist.join(DIC_DIR_NAME);
    if !dic_dir.try_exists()? {
        download_open_jtalk_dict(dist.as_ref()).await?;
        ensure!(dic_dir.exists(), "`{dic_dir}` does not exist");
    }

    copy_onnxruntime(out_dir.as_ref(), dist)?;

    create_sample_voice_model_file(out_dir, dist)?;

    generate_example_data_json(dist.as_ref())?;

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/typing.rs");

    generate_c_api_rs_bindings(out_dir)
}

fn create_sample_voice_model_file(out_dir: &Utf8Path, dist: &Utf8Path) -> anyhow::Result<()> {
    const SRC: &str = "../../model/sample.vvm";

    let files = fs_err::read_dir(SRC)?
        .map(|entry| {
            let entry = entry?;
            let md = entry.metadata()?;
            ensure!(!md.is_dir(), "directory in {SRC}");
            let mtime = md.modified()?;
            let name = entry
                .file_name()
                .into_string()
                .map_err(|name| anyhow!("{name:?}"))?;
            Ok((name, entry.path(), mtime))
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    let output_dir = &dist.join("model");
    let output_file = &output_dir.join("sample.vvm");

    let up_to_date = fs_err::metadata(output_file)
        .and_then(|md| md.modified())
        .map(|t1| files.iter().all(|&(_, _, t2)| t1 >= t2));
    let up_to_date = match up_to_date {
        Ok(p) => p,
        Err(e) if e.kind() == io::ErrorKind::NotFound => false,
        Err(e) => return Err(e.into()),
    };

    if !up_to_date {
        let mut zip = ZipWriter::new(Cursor::new(vec![]));
        for (name, path, _) in files {
            let content = &fs_err::read(path)?;
            zip.start_file(name, FileOptions::default().compression_level(Some(0)))?;
            zip.write_all(content)?;
        }
        let zip = zip.finish()?;
        fs_err::create_dir_all(output_dir)?;
        fs_err::write(output_file, zip.get_ref())?;
    }

    fs_err::write(
        out_dir.join("sample_voice_model_file.rs"),
        formatdoc! {"
            pub const SAMPLE_VOICE_MODEL_FILE_PATH: &::std::primitive::str = {output_file:?};

            const SAMPLE_VOICE_MODEL_FILE_C_PATH: &::std::ffi::CStr = c{output_file:?};
            const VV_MODELS_ROOT_DIR: &::std::primitive::str = {output_dir:?};
            ",
        },
    )?;
    println!("cargo:rerun-if-changed={SRC}");
    Ok(())
}

fn copy_onnxruntime(out_dir: &Path, dist: &Utf8Path) -> anyhow::Result<()> {
    use std::env::consts::{DLL_PREFIX, DLL_SUFFIX};

    let cargo_metadata::Metadata {
        target_directory, ..
    } = MetadataCommand::new()
        .manifest_path(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
        .exec()?;

    const VERSION: &str = include_str!("../voicevox_core/onnxruntime-version.txt");
    let filename = &if cfg!(target_os = "linux") {
        format!("libonnxruntime.so.{VERSION}")
    } else if cfg!(any(target_os = "macos", target_os = "ios")) {
        format!("libonnxruntime.{VERSION}.dylib")
    } else {
        format!("{DLL_PREFIX}onnxruntime{DLL_SUFFIX}")
    };
    let src = &target_directory.join("debug").join(filename);
    let dst_dir = &dist.join("lib");
    let dst = &dst_dir.join(filename);
    fs_err::create_dir_all(dst_dir)?;
    fs_err::copy(src, dst)?;
    println!("cargo:rerun-if-changed={src}");

    fs_err::write(out_dir.join("onnxruntime-dylib-path.txt"), dst.as_str())?;

    Ok(())
}

/// OpenJTalkの辞書をダウンロードして展開する。
async fn download_open_jtalk_dict(dist: &Path) -> anyhow::Result<()> {
    let download_url = format!(
        "https://github.com/r9y9/open_jtalk/releases/download/v1.11.1/{DIC_DIR_NAME}.tar.gz"
    );

    let res = reqwest::get(&download_url).await?;
    ensure!(res.status() == 200, "{}", res.status());

    let bytes = res.bytes().await?;
    let dict_tar = GzDecoder::new(&*bytes);

    let mut dict_archive = Archive::new(dict_tar);
    dict_archive.unpack(dist)?;
    Ok(())
}

/// テストデータのJSONを生成する。
fn generate_example_data_json(dist: &Path) -> anyhow::Result<()> {
    let test_data = typing::ExampleData {
        speaker_id: 0,

        duration: typing::DurationExampleData {
            length: 8,
            // 「t e s u t o」
            phoneme_vector: vec![0, 37, 14, 35, 6, 37, 30, 0],
            result: vec![
                0.9537022,
                0.046877652,
                0.11338878,
                0.06429571,
                0.07507616,
                0.08266081,
                0.1571679,
                0.64980185,
            ],
        },
        intonation: typing::IntonationExampleData {
            length: 5,

            vowel_phoneme_vector: vec![0, 14, 6, 30, 0],
            consonant_phoneme_vector: vec![-1, 37, 35, 37, -1],
            start_accent_vector: vec![0, 1, 0, 0, 0],
            end_accent_vector: vec![0, 1, 0, 0, 0],

            start_accent_phrase_vector: vec![0, 1, 0, 0, 0],

            end_accent_phrase_vector: vec![0, 0, 0, 1, 0],

            result: vec![5.0591826, 5.905218, 5.846999, 5.565851, 5.528879],
        },
        decode: typing::DecodeExampleData {
            f0_length: 69,
            phoneme_size: 45,
            f0_vector: {
                let mut f0 = [0.; 69];
                f0[9..24].fill(5.905218);
                f0[37..60].fill(5.565851);
                f0.to_vec()
            },
            phoneme_vector: {
                let mut phoneme = [0.; 45 * 69];
                let mut set_one = |index, range| {
                    for i in range {
                        phoneme[(i * 45 + index) as usize] = 1.;
                    }
                };
                set_one(0, 0..9);
                set_one(37, 9..13);
                set_one(14, 13..24);
                set_one(35, 24..30);
                set_one(6, 30..37);
                set_one(37, 37..45);
                set_one(30, 45..60);
                set_one(0, 60..69);
                phoneme.to_vec()
            },
        },
        intermediate: typing::IntermediateExampleData {
            f0_length: 69,
            phoneme_size: 45,
            feature_dim: 80,
            margin_width: 14,
            f0_vector: {
                let mut f0 = [0.; 69];
                f0[9..24].fill(5.905218);
                f0[37..60].fill(5.565851);
                f0.to_vec()
            },
            phoneme_vector: {
                let mut phoneme = [0.; 45 * 69];
                let mut set_one = |index, range| {
                    for i in range {
                        phoneme[(i * 45 + index) as usize] = 1.;
                    }
                };
                set_one(0, 0..9);
                set_one(37, 9..13);
                set_one(14, 13..24);
                set_one(35, 24..30);
                set_one(6, 30..37);
                set_one(37, 37..45);
                set_one(30, 45..60);
                set_one(0, 60..69);
                phoneme.to_vec()
            },
        },
    };

    fs_err::write(
        dist.join("example_data.json"),
        serde_json::to_string(&test_data)?,
    )?;

    Ok(())
}

fn generate_c_api_rs_bindings(out_dir: &Utf8Path) -> anyhow::Result<()> {
    static C_BINDINGS_PATH: &str = "../voicevox_core_c_api/include/voicevox_core.h";
    static ADDITIONAL_C_BINDINGS_PATH: &str = "./compatible_engine.h";

    bindgen::Builder::default()
        .header(C_BINDINGS_PATH)
        .header(ADDITIONAL_C_BINDINGS_PATH)
        // we test for `--feature load-onnxruntime`
        .clang_arg("-DVOICEVOX_LOAD_ONNXRUNTIME=")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .dynamic_library_name("CApi")
        .generate()?
        .write_to_file(out_dir.join("c_api.rs"))?;
    println!("cargo:rerun-if-changed={C_BINDINGS_PATH}");
    println!("cargo:rerun-if-changed={ADDITIONAL_C_BINDINGS_PATH}");
    Ok(())
}
