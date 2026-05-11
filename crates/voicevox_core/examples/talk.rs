use std::fs;

use anyhow::Context as _;
use camino::Utf8PathBuf;
use clap::Parser;
use voicevox_core::blocking::{Onnxruntime, OpenJtalk, Synthesizer, VoiceModelFile};

use const_format::formatcp;

const VOICEXVOX_CORE_DIR: &str = "./voicevox_core";
const DEFAULT_ONNXRUNTIME: &str = formatcp!(
    "{VOICEXVOX_CORE_DIR}/onnxruntime/lib/{}",
    Onnxruntime::LIB_VERSIONED_FILENAME,
);
const DEFAULT_MODEL: &str = formatcp!("{VOICEXVOX_CORE_DIR}/models/vvms/0.vvm");
const DEFAULT_DICT: &str = formatcp!("{VOICEXVOX_CORE_DIR}/dict/open_jtalk_dic_utf_8-1.11");

/// テキスト音声合成を行うサンプルコードです。
///
/// `cargo run -p downloader` でリソースをダウンロードしてから実行してください。
#[derive(Parser)]
struct Args {
    /// 合成するテキスト
    #[arg(
        long,
        default_value = "この音声は、ボイスボックスを使用して、出力されています。"
    )]
    text: String,

    /// 出力するWAVファイルのパス
    #[arg(long, default_value = "./output.wav")]
    out: Utf8PathBuf,

    /// 読み込むVVMファイルのパス
    #[arg(long, default_value = DEFAULT_MODEL)]
    vvm: Utf8PathBuf,

    /// ONNX Runtimeのライブラリのパス
    #[arg(long, default_value = DEFAULT_ONNXRUNTIME)]
    onnxruntime: Utf8PathBuf,

    /// Open JTalkの辞書ディレクトリ
    #[arg(long, default_value = DEFAULT_DICT)]
    dict_dir: Utf8PathBuf,

    /// 話者名
    #[arg(long, default_value = "ずんだもん")]
    character: String,

    /// スタイル名
    #[arg(long, default_value = "ノーマル")]
    style: String,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // ONNX Runtimeのロード
    let ort = Onnxruntime::load_once()
        .filename(args.onnxruntime.into_os_string())
        .perform()
        .context("ONNX Runtimeのロードに失敗しました")?;

    // Synthesizerの構築
    let ojt = OpenJtalk::new(args.dict_dir).context("Open JTalk辞書のロードに失敗しました")?;
    let synth = Synthesizer::builder(ort)
        .text_analyzer(ojt)
        .build()
        .context("Synthesizerの構築に失敗しました")?;

    let model = VoiceModelFile::open(args.vvm).context("音声モデルの読み込みに失敗しました")?;
    synth
        .load_voice_model(&model)
        .perform()
        .context("音声モデルのロードに失敗しました")?;

    // スタイルIDの取得
    let style_id = synth
        .metas()
        .iter()
        .find(|m| m.name == args.character)
        .and_then(|m| m.styles.iter().find(|s| s.name == args.style))
        .map(|s| s.id)
        .with_context(|| {
            format!(
                "キャラクター \"{}\" のスタイル \"{}\" が見つかりませんでした",
                args.character, args.style
            )
        })?;

    eprintln!("合成中...");
    let wav = synth
        .tts(&args.text, style_id)
        .perform()
        .context("音声合成に失敗しました")?;

    fs::write(&args.out, wav).context("出力ファイルの書き込みに失敗しました")?;
    eprintln!("Saved to {:?}", args.out);

    Ok(())
}
