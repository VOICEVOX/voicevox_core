use std::{
    env, path::{Path, PathBuf}
};

use anyhow::ensure;
use camino::Utf8PathBuf;
use flate2::read::GzDecoder;
use tar::Archive;

#[path = "src/typing.rs"]
mod typing;

const DIC_DIR_NAME: &str = "open_jtalk_dic_utf_8-1.11";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut dist = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    dist.push("data");

    let dic_dir = dist.join(DIC_DIR_NAME);

    if !dic_dir.try_exists()? {
        download_open_jtalk_dict(&dist).await?;
        ensure!(dic_dir.exists(), "`{}` does not exist", dic_dir.display());
    }

    generate_example_data_json(&dist)?;

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/typing.rs");

    generate_c_api_rs_bindings()
}

/// OpenJTalkの辞書をダウンロードして展開する。
async fn download_open_jtalk_dict(dist: &Path) -> anyhow::Result<()> {
    let download_url = format!(
        "https://github.com/r9y9/open_jtalk/releases/download/v1.11.1/{DIC_DIR_NAME}.tar.gz"
    );

    let res = reqwest::get(&download_url).await?;
    ensure!(res.status().is_success(), "{}", res.status());

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
    };

    fs_err::write(
        dist.join("example_data.json"),
        serde_json::to_string(&test_data)?,
    )?;

    Ok(())
}

fn generate_c_api_rs_bindings() -> anyhow::Result<()> {
    static C_BINDINGS_PATH: &str = "../voicevox_core_c_api/include/voicevox_core.h";
    static ADDITIONAL_C_BINDINGS_PATH: &str = "./compatible_engine.h";

    let out_dir = Utf8PathBuf::from(env::var("OUT_DIR").unwrap());
    bindgen::Builder::default()
        .header(C_BINDINGS_PATH)
        .header(ADDITIONAL_C_BINDINGS_PATH)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .dynamic_library_name("CApi")
        .generate()?
        .write_to_file(out_dir.join("c_api.rs"))?;
    println!("cargo:rerun-if-changed={C_BINDINGS_PATH}");
    println!("cargo:rerun-if-changed={ADDITIONAL_C_BINDINGS_PATH}");
    Ok(())
}
