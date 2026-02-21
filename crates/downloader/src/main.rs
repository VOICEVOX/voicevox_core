use std::{
    borrow::Cow,
    collections::{BTreeSet, HashSet},
    env,
    future::{self, Future},
    io::{self, Cursor, IsTerminal as _, Read, Write as _},
    iter,
    num::NonZero,
    path::{Path, PathBuf},
    str::FromStr,
    sync::{Arc, LazyLock},
    time::Duration,
};

use anyhow::{Context as _, anyhow, bail};
use bytes::Bytes;
use clap::{Parser as _, ValueEnum, crate_version};
use easy_ext::ext;
use flate2::read::GzDecoder;
use futures_core::Stream;
use futures_util::{
    StreamExt as _, TryStreamExt as _,
    future::OptionFuture,
    stream::{FuturesOrdered, FuturesUnordered},
};
use heck::ToSnakeCase as _;
use indexmap::IndexMap;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use indoc::{formatdoc, indoc};
use itertools::Itertools as _;
use octocrab::{
    Octocrab,
    models::{
        AssetId,
        repos::{Asset, Release},
    },
    repos::RepoHandler,
};
use rayon::iter::{IntoParallelIterator as _, ParallelIterator as _};
use semver::{Version, VersionReq};
use strum::{Display, IntoStaticStr};
use tokio::task::{JoinError, JoinSet};
use tracing::{error, info, warn};
use unicode_width::UnicodeWidthStr as _;
use zip::ZipArchive;

const DEFAULT_OUTPUT: &str = if cfg!(windows) {
    r".\voicevox_core"
} else {
    "./voicevox_core"
};

const C_API_LIB_NAME: &str = "voicevox_core";
const DEFAULT_C_API_REPO: &str = "VOICEVOX/voicevox_core";
const DEFAULT_ONNXRUNTIME_BUILDER_REPO: &str = "VOICEVOX/onnxruntime-builder";
const DEFAULT_ADDITIONAL_LIBRARIES_REPO: &str = "VOICEVOX/voicevox_additional_libraries";
const DEFAULT_MODELS_REPO: &str = "VOICEVOX/voicevox_vvm";

const ONNXRUNTIME_TERMS_NAME: &str = "VOICEVOX ONNX Runtime 利用規約";

static SUPPORTED_MODELS_VERSIONS: LazyLock<VersionReq> =
    LazyLock::new(|| ">=0.16,<0.17".parse().unwrap());
const MODELS_README_FILENAME: &str = "README.txt";
const MODELS_DIR_NAME: &str = "vvms";
const MODELS_TERMS_NAME: &str = "VOICEVOX 音声モデル 利用規約";
const MODELS_TERMS_FILE: &str = "TERMS.txt";

static OPEN_JTALK_DIC_REPO: LazyLock<RepoName> = LazyLock::new(|| RepoName {
    owner: "r9y9".to_owned(),
    repo: "open_jtalk".to_owned(),
});
const OPEN_JTALK_DIC_TAG: &str = "v1.11.1";
const OPEN_JTALK_DIC_FILE: &str = "open_jtalk_dic_utf_8-1.11.tar.gz";

static PROGRESS_STYLE0: LazyLock<ProgressStyle> =
    LazyLock::new(|| ProgressStyle::with_template("{prefix}").unwrap());
static PROGRESS_STYLE1: LazyLock<ProgressStyle> = LazyLock::new(|| {
    ProgressStyle::with_template(
        "{prefix:55} {bytes:>11} {bytes_per_sec:>13} {elapsed_precise} {bar} {percent:>3}%",
    )
    .unwrap()
});
static PROGRESS_STYLE2: LazyLock<ProgressStyle> =
    LazyLock::new(|| ProgressStyle::with_template("{prefix:55} {spinner} {msg}").unwrap());

#[derive(clap::Parser)]
#[command(
    name("VOICEVOX CORE"),
    version(concat!(crate_version!(), " downloader")),
    about("簡潔な説明を見るには`-h`、詳細な説明を見るには`--help`を使ってください。"),
    after_long_help(formatdoc! {"
          {targets_section_header}

            `--only`や`--exclude`で特に指定しない場合、ダウンローダーは次のすべてをダウンロードします。

          {targets_section_target_values}

          {github_token_section_header}

            環境変数{env_gh_token}または{env_github_token}からGitHubの認証トークンを設定することができます。
            両方設定されている場合は{env_gh_token}が優先されます。
            トークン無しのアクセスには低いレートリミットが課せられているため、設定することをおすすめします。

                GH_TOKEN=$(gh auth token) download …

          {examples_section_header}

            デフォルト(CPU 版)をダウンロードする場合:

                download

            DirectML 版をダウンロードする場合:

                download --devices directml

            CUDA 版をダウンロードする場合:

                download --devices cuda

            一部の音声モデル（VVMファイル）だけダウンロードする場合:

                download --models-pattern 0.vvm # 0.vvmのみダウンロード

                download --models-pattern '[0-9]*.vvm' # トーク用VVMに絞り、ソング用VVMをダウンロードしないように
          ",
          targets_section_header = color_print::cstr!("<s><u>Targets:</u></s>"),
          targets_section_target_values = DownloadTarget::value_variants()
              .iter()
              .map(|download_target| formatdoc! {"
                  • {download_target} (展開先: {{output}}/{dir_name}/):
                          {description}",
                  download_target = color_print::cformat!("<s>{download_target}</s>"),
                  dir_name = download_target.dir_name(),
                  description = download_target.description(),
              })
              .join("\n\n")
              .lines()
              .map(|line| format!("  {line}"))
              .join("\n"),
          github_token_section_header = color_print::cstr!(
              "<s><u>GitHub Authentication Token:</u></s>",
          ),
          env_gh_token = color_print::cstr!("<s>GH_TOKEN</s>"),
          env_github_token = color_print::cstr!("<s>GITHUB_TOKEN</s>"),
          examples_section_header = color_print::cstr!("<s><u>Examples:</u></s>"),
    })
)]
struct Args {
    /// ダウンロード対象を限定する
    #[arg(
        long,
        num_args(1..),
        value_name("TARGET"),
        conflicts_with_all(["exclude", "min"]),
        long_help(indoc! {"
            ダウンロード対象を限定する。

            ダウンロード対象の詳細はTargetsの章で説明。",
        })
    )]
    only: Vec<DownloadTarget>,

    /// ダウンロード対象を除外する
    #[arg(
        long,
        num_args(1..),
        value_name("TARGET"),
        conflicts_with("min"),
        long_help(indoc! {"
            ダウンロード対象を除外する。

            ダウンロード対象の詳細はTargetsの章で説明。",
        })
    )]
    exclude: Vec<DownloadTarget>,

    /// `--only c-api`のエイリアス
    #[arg(
        long,
        conflicts_with("additional_libraries_version"),
        long_help("`--only c-api`のエイリアス。")
    )]
    min: bool,

    /// 出力先の指定
    #[arg(
        short,
        long,
        value_name("DIRECTORY"),
        default_value(DEFAULT_OUTPUT),
        long_help("出力先の指定。")
    )]
    output: PathBuf,

    /// ダウンロードするVOICEVOX CORE C APIのバージョンの指定
    #[arg(
        long,
        value_name("GIT_TAG_OR_LATEST"),
        default_value("latest"),
        long_help("ダウンロードするVOICEVOX CORE C APIのバージョンの指定。")
    )]
    c_api_version: String,

    /// ダウンロードするONNX Runtimeのバージョンの指定
    #[arg(
        long,
        value_name("GIT_TAG_OR_LATEST"),
        default_value("latest"),
        long_help("ダウンロードするONNX Runtimeのバージョンの指定。")
    )]
    onnxruntime_version: String,

    /// 追加でダウンロードするライブラリのバージョン
    #[arg(
        long,
        value_name("GIT_TAG_OR_LATEST"),
        default_value("latest"),
        long_help("追加でダウンロードするライブラリのバージョン。")
    )]
    additional_libraries_version: String,

    #[arg(
        long,
        value_name("SEMVER"),
        help(format!(
            "VOICEVOX音声モデル (`models`)のバージョン。\
             省略時は`{SUPPORTED_MODELS_VERSIONS}`のうちpre-releaseではない最新",
            SUPPORTED_MODELS_VERSIONS = *SUPPORTED_MODELS_VERSIONS,
        )),
        long_help(format!(
            "VOICEVOX音声モデル (`models`)のバージョン。\n\
             \n\
             省略した場合は{SUPPORTED_MODELS_VERSIONS}のうち、pre-releaseではない最新のものになる。",
            SUPPORTED_MODELS_VERSIONS = color_print::cformat!(
                "<s>{SUPPORTED_MODELS_VERSIONS}</s>",
                SUPPORTED_MODELS_VERSIONS = *SUPPORTED_MODELS_VERSIONS,
            ),
        ))
    )]
    models_version: Option<Version>,

    /// ダウンロードするVVMファイルのファイル名パターン
    #[arg(
        long,
        value_name("GLOB"),
        default_value("*"),
        long_help("ダウンロードするVVMファイルのファイル名パターン。")
    )]
    models_pattern: glob::Pattern,

    /// ダウンロードするデバイスを指定する
    #[arg(
        value_enum,
        long,
        num_args(1..),
        default_value(<&str>::from(Device::default())),
        long_help("ダウンロードするデバイスを指定する。")
    )]
    devices: Vec<Device>,

    /// ダウンロードするcpuのアーキテクチャを指定する
    #[arg(
        value_enum,
        long,
        default_value(CpuArch::default_opt().map(<&str>::from)),
        long_help("ダウンロードするcpuのアーキテクチャを指定する。")
    )]
    cpu_arch: CpuArch,

    /// ダウンロードする対象のOSを指定する
    #[arg(
        value_enum,
        long,
        default_value(Os::default_opt().map(<&str>::from)),
        long_help("ダウンロードする対象のOSを指定する。")
    )]
    os: Os,

    /// ダウンロードにおける試行回数。'0'か'inf'で無限にリトライ
    #[arg(
        short,
        long,
        value_name("NUMBER"),
        default_value("5"),
        long_help(formatdoc! {"
            ダウンロードにおける試行回数。'0'か'inf'で無限にリトライ。

            現段階では以下に示す挙動をする。

            • 各試行は{DOWNLOAD_TARGET}単位で行われる。
              ダウンロードしたzipやtgzの解凍に失敗してもリトライが行われる。
              また{DOWNLOAD_TARGET_MODEL}の場合、どれか一つのVVMのダウンロードに失敗すると
              他のVVMも全部まとめてリトライが行われる。
            • プログレスバーを出す前の段階でエラーが発生した場合、リトライは行われない。

            これらの挙動は将来的に変更される予定であり、議論は
            https://github.com/VOICEVOX/voicevox_core/issues/1127
            で行われている。",
            DOWNLOAD_TARGET = color_print::cstr!("<s><<TARGET>></>"),
            DOWNLOAD_TARGET_MODEL = color_print::cstr!("<s>models</>"),
        })
    )]
    tries: Tries,

    /// VOICEVOX CORE C API (`c-api`)のリポジトリ
    #[arg(
        long,
        value_name("REPOSITORY"),
        default_value(DEFAULT_C_API_REPO),
        long_help("VOICEVOX CORE C API (`c-api`)のリポジトリ。")
    )]
    c_api_repo: RepoName,

    /// (VOICEVOX) ONNX Runtime (`onnxruntime`)のリポジトリ
    #[arg(
        long,
        value_name("REPOSITORY"),
        default_value(DEFAULT_ONNXRUNTIME_BUILDER_REPO),
        long_help("(VOICEVOX) ONNX Runtime (`onnxruntime`)のリポジトリ。")
    )]
    onnxruntime_builder_repo: RepoName,

    /// 追加でダウンロードするライブラリ (`additional-libraries`)のリポジトリ
    #[arg(
        long,
        value_name("REPOSITORY"),
        default_value(DEFAULT_ADDITIONAL_LIBRARIES_REPO),
        long_help("追加でダウンロードするライブラリ (`additional-libraries`)のリポジトリ。")
    )]
    additional_libraries_repo: RepoName,

    /// VOICEVOX音声モデル (`models`)のリポジトリ
    #[arg(
        long,
        value_name("REPOSITORY"),
        default_value(DEFAULT_MODELS_REPO),
        long_help("VOICEVOX音声モデル (`models`)のリポジトリ。")
    )]
    models_repo: RepoName,
}

#[derive(ValueEnum, Display, IntoStaticStr, Clone, Copy, PartialEq, Eq, Hash)]
#[strum(serialize_all = "kebab-case")]
enum DownloadTarget {
    CApi,
    Onnxruntime,
    AdditionalLibraries,
    Models,
    Dict,
}

impl DownloadTarget {
    fn description(self) -> &'static str {
        match self {
            Self::CApi => "VOICEVOX CORE C APIのビルド済みバイナリおよびその利用規約ファイル等。",
            Self::Onnxruntime => "(VOICEVOX) ONNX Runtime。",
            Self::AdditionalLibraries => "`--devices`で指定したDirectMLやCUDA。",
            Self::Models => "VOICEVOX音声モデル（VVMファイル）。",
            Self::Dict => "Open JTalkのシステム辞書。",
        }
    }

    fn dir_name(self) -> String {
        <&str>::from(self).to_snake_case()
    }
}

#[derive(
    Default, ValueEnum, Display, IntoStaticStr, Clone, Copy, PartialEq, Eq, PartialOrd, Ord,
)]
#[strum(serialize_all = "kebab-case")]
enum Device {
    #[default]
    Cpu,
    Cuda,
    Directml,
}

#[derive(ValueEnum, Display, IntoStaticStr, Clone, Copy, PartialEq)]
#[strum(serialize_all = "kebab-case")]
enum CpuArch {
    X86,
    X64,
    Arm64,
}

impl CpuArch {
    fn default_opt() -> Option<Self> {
        match env::consts::ARCH {
            "x86_64" => Some(Self::X64),
            "aarch64" => Some(Self::Arm64),
            _ => None,
        }
    }
}

#[derive(ValueEnum, Display, IntoStaticStr, Clone, Copy, PartialEq)]
#[strum(serialize_all = "kebab-case")]
enum Os {
    Windows,
    Linux,
    Osx,
    Android,
    Ios,
}

impl Os {
    fn default_opt() -> Option<Self> {
        match env::consts::OS {
            "windows" => Some(Self::Windows),
            "linux" => Some(Self::Linux),
            "macos" => Some(Self::Osx),
            "android" => Some(Self::Android),
            "ios" => Some(Self::Ios),
            _ => None,
        }
    }
}

#[derive(parse_display::FromStr, parse_display::Display, Clone)]
#[from_str(regex = "(?<owner>[a-zA-Z0-9_-]+)/(?<repo>[a-zA-Z0-9_-]+)")]
#[display("{owner}/{repo}")]
struct RepoName {
    owner: String,
    repo: String,
}

#[derive(Clone, Copy)]
enum Tries {
    Finite(NonZero<u32>),
    Infinite,
}

impl FromStr for Tries {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" | "inf" => Ok(Self::Infinite),
            s => s
                .parse()
                .map(Self::Finite)
                .map_err(|_| "must be a positive integer or `inf`"),
        }
    }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    setup_logger();

    let Args {
        only,
        exclude,
        min,
        output,
        c_api_version,
        onnxruntime_version,
        additional_libraries_version,
        models_version,
        models_pattern,
        devices,
        cpu_arch,
        os,
        tries,
        c_api_repo,
        onnxruntime_builder_repo,
        models_repo,
        additional_libraries_repo,
    } = Args::parse();
    let devices = devices.into_iter().collect::<BTreeSet<_>>();

    let targets: HashSet<_> = if !only.is_empty() {
        assert!(exclude.is_empty() && !min);
        only.into_iter().collect()
    } else if !exclude.is_empty() {
        assert!(!min);
        DownloadTarget::value_variants()
            .iter()
            .copied()
            .filter(|t| !exclude.contains(t))
            .collect()
    } else if min {
        [DownloadTarget::CApi].into()
    } else {
        DownloadTarget::value_variants().iter().copied().collect()
    };

    if os == Os::Ios && targets.contains(&DownloadTarget::CApi) {
        bail!(
            "`--os ios`の場合、`c-api`をダウンロード対象に含めることはできません。\
            `--only <TARGET>...`または`--exclude <TARGET>...`で`c-api`をダウンロード対象から\
            除外してください",
        );
    }
    if !targets.contains(&DownloadTarget::CApi) {
        if c_api_version != "latest" {
            warn!(
                "`--c-api-version={c_api_version}`が指定されていますが、`c-api`はダウンロード対象から\
                 除外されています",
            );
        }
        if c_api_repo.to_string() != DEFAULT_C_API_REPO {
            warn!(
                "`--c-api-repo={c_api_repo}`が指定されていますが、`c-api`はダウンロード対象\
                 から除外されています",
            );
        }
    }
    if !targets.contains(&DownloadTarget::AdditionalLibraries) {
        if additional_libraries_version != "latest" {
            warn!(
                "`--additional-libraries-version={additional_libraries_version}`が指定されています\
                 が、`additional-libraries-version`はダウンロード対象から除外されています",
            );
        }
        if additional_libraries_repo.to_string() != DEFAULT_ADDITIONAL_LIBRARIES_REPO {
            warn!(
                "`--additional-libraries-repo={additional_libraries_repo}`が指定されていますが、\
                 `additional-libraries-version`はダウンロード対象から除外されています",
            );
        }
        if devices == [Device::Cpu].into() {
            warn!(
                "`--devices`が指定されていない、もしくは`--devices=cpu`が指定されていますが、\
                 `additional-libraries-version`はダウンロード対象から除外されています",
            );
        }
    }

    // FIXME: `--models-repo`に対しても警告を出す
    // FIXME: あと`--models-pattern`にも
    if !targets.contains(&DownloadTarget::Models) && models_pattern.as_str() != "*" {
        warn!(
            "`--models-pattern={models_pattern}`が指定されていますが、`models`はダウンロード対象\
             から除外されています",
            models_pattern = models_pattern.as_str(),
        );
    }
    if !targets.contains(&DownloadTarget::Models)
        && let Some(models_version) = &models_version
    {
        warn!(
            "`--models-version={models_version}`が指定されていますが、`models`はダウンロード対象\
             から除外されています",
        );
    }

    if let Some(models_version) = &models_version
        && !SUPPORTED_MODELS_VERSIONS.matches(models_version)
    {
        warn!(
            "サポートされているバージョンは{SUPPORTED_MODELS_VERSIONS}です: {models_version}",
            SUPPORTED_MODELS_VERSIONS = *SUPPORTED_MODELS_VERSIONS,
        );
    }

    let octocrab = &octocrab()?;

    let c_api = OptionFuture::from(targets.contains(&DownloadTarget::CApi).then(|| {
        find_gh_asset(octocrab, &c_api_repo, &c_api_version, |tag, _| {
            if os == Os::Ios {
                unreachable!("should have been denied beforehand");
            }
            Ok(format!("{C_API_LIB_NAME}-{os}-{cpu_arch}-{tag}.zip"))
        })
    }))
    .await
    .transpose()?;

    let onnxruntime =
        OptionFuture::from(targets.contains(&DownloadTarget::Onnxruntime).then(|| {
            find_gh_asset(
                octocrab,
                &onnxruntime_builder_repo,
                &onnxruntime_version,
                |_, body| {
                    let body = body.with_context(|| "リリースノートがありません")?;
                    find_onnxruntime(body, os, cpu_arch, &devices)
                },
            )
        }))
        .await
        .transpose()?;

    let models = OptionFuture::from(targets.contains(&DownloadTarget::Models).then(|| {
        find_models(
            octocrab,
            &models_repo,
            models_version.as_ref(),
            &models_pattern,
        )
    }))
    .await
    .transpose()?;

    let dict = OptionFuture::from(targets.contains(&DownloadTarget::Dict).then(|| {
        find_gh_asset(
            octocrab,
            &OPEN_JTALK_DIC_REPO,
            OPEN_JTALK_DIC_TAG,
            |_, _| Ok(OPEN_JTALK_DIC_FILE.to_owned()),
        )
    }))
    .await
    .transpose()?;

    ensure_confirmation(
        &iter::chain(
            models
                .as_ref()
                .map(|ModelsWithTerms { terms, .. }| &**terms)
                .map(|terms| (MODELS_TERMS_NAME, terms)),
            onnxruntime
                .as_ref()
                .map(extract_voicevox_onnxruntime_terms)
                .transpose()?
                .flatten()
                .as_deref()
                .map(|terms| (ONNXRUNTIME_TERMS_NAME, terms)),
        )
        .collect(),
    )?;

    let additional_libraries = devices
        .iter()
        .filter(|&&device| device != Device::Cpu)
        .map(|&device| {
            find_gh_asset(
                octocrab,
                &additional_libraries_repo,
                &additional_libraries_version,
                move |_, _| {
                    Ok({
                        let device = match device {
                            Device::Cpu => unreachable!(),
                            Device::Cuda => "CUDA",
                            Device::Directml => "DirectML",
                        };
                        format!("{device}-{os}-{cpu_arch}.zip")
                    })
                },
            )
        })
        .collect::<FuturesOrdered<_>>()
        .try_collect::<Vec<_>>()
        .await?;

    info!("対象OS: {os}");
    info!("対象CPUアーキテクチャ: {cpu_arch}");
    info!(
        "ダウンロードデバイスタイプ: {}",
        devices.iter().format(", "),
    );
    if let Some(GhAsset { tag, .. }) = &c_api {
        info!("ダウンロード{C_API_LIB_NAME}バージョン: {tag}");
    }
    if let Some(GhAsset { tag, .. }) = &onnxruntime {
        info!("ダウンロードONNX Runtimeバージョン: {tag}");
    }
    if !additional_libraries.is_empty() {
        info!(
            "ダウンロード追加ライブラリバージョン: {}",
            additional_libraries
                .iter()
                .map(|GhAsset { tag, .. }| tag)
                .format(", "),
        );
    }
    if let Some(ModelsWithTerms { tag, .. }) = &models {
        info!("ダウンロードモデルバージョン: {tag}");
    }
    if let Some(GhAsset { tag, .. }) = &dict {
        assert_eq!(OPEN_JTALK_DIC_TAG, tag);
        info!("ダウンロードOpen JTalk辞書バージョン: {OPEN_JTALK_DIC_TAG}");
    }

    let progresses = MultiProgress::new();

    let mut tasks = JoinSet::new();

    if let Some(c_api) = c_api {
        tasks.spawn(download_and_extract_from_gh(
            c_api,
            Stripping::FirstDir,
            output.join(DownloadTarget::CApi.dir_name()),
            &progresses,
            tries,
        )?);
    }
    if let Some(onnxruntime) = onnxruntime {
        tasks.spawn(download_and_extract_from_gh(
            onnxruntime,
            Stripping::FirstDir,
            output.join(DownloadTarget::Onnxruntime.dir_name()),
            &progresses,
            tries,
        )?);
    }
    if targets.contains(&DownloadTarget::AdditionalLibraries) {
        for additional_libraries in additional_libraries {
            tasks.spawn(download_and_extract_from_gh(
                additional_libraries,
                Stripping::FirstDir,
                output.join(DownloadTarget::AdditionalLibraries.dir_name()),
                &progresses,
                tries,
            )?);
        }
    }
    if let Some(models) = models {
        tasks.spawn(
            download_models(
                models,
                output.join(DownloadTarget::Models.dir_name()),
                &progresses,
                tries,
            )
            .await?,
        );
    }
    if let Some(dict) = dict {
        tasks.spawn(download_and_extract_from_gh(
            dict,
            Stripping::None,
            output.join(DownloadTarget::Dict.dir_name()),
            &progresses,
            tries,
        )?);
    }

    while let Some(result) = tasks.join_next().await {
        result??;
    }

    info!("全ての必要なファイルダウンロードが完了しました");
    Ok(())
}

fn setup_logger() {
    tracing_subscriber::fmt()
        .with_env_filter(format!("error,{}=info", env!("CARGO_CRATE_NAME")))
        .with_writer(io::stderr)
        .with_target(false)
        .without_time()
        .init();
}

fn octocrab() -> octocrab::Result<Arc<Octocrab>> {
    let mut octocrab = Octocrab::builder();
    if let Ok(github_token) = env::var("GH_TOKEN").or_else(|_| env::var("GITHUB_TOKEN")) {
        octocrab = octocrab.personal_token(github_token);
    }
    octocrab.build().map(Arc::new)
}

async fn retry<F>(tries: Tries, mut f: F) -> anyhow::Result<()>
where
    F: AsyncFnMut() -> anyhow::Result<()>,
{
    match tries {
        Tries::Infinite => loop {
            if let Ok(o) = f().await {
                return Ok(o);
            }
        },
        Tries::Finite(nonzero) => {
            let mut attempt = async || {
                f().await.map_err(|err| {
                    err.chain().skip(1).fold(format!("- {err}"), |msg, cause| {
                        format!("{msg}\n  Caused by: {cause}")
                    })
                })
            };

            let mut result = attempt().await;
            for _ in 0..nonzero.get() - 1 {
                if let Err(err1) = result {
                    result = attempt().await.map_err(|err2| format!("{err1}\n{err2}"));
                } else {
                    break;
                }
            }
            result.map_err(|causes| {
                anyhow::Error::msg(causes)
                    .context(format!("{nonzero}回のダウンロード試行がすべて失敗しました"))
            })
        }
    }
}

async fn find_gh_asset(
    octocrab: &Arc<Octocrab>,
    repo: &RepoName,
    git_tag_or_latest: &str,
    asset_name: impl FnOnce(
        &str,         // タグ名
        Option<&str>, // リリースノートの内容
    ) -> anyhow::Result<String>,
) -> anyhow::Result<GhAsset> {
    let Release {
        html_url,
        tag_name,
        body,
        assets,
        ..
    } = {
        let repos = octocrab.repos(&repo.owner, &repo.repo);
        let releases = repos.releases();
        match git_tag_or_latest {
            "latest" => releases.get_latest().await,
            tag => releases.get_by_tag(tag).await,
        }?
    };

    let asset_name = asset_name(&tag_name, body.as_deref()).with_context(|| {
        format!(
            "`{repo}`の`{tag_name}`の中から条件に合致するビルドを見つけることができませんでした",
        )
    })?;
    let Asset { id, name, size, .. } = assets
        .into_iter()
        .find(|Asset { name, .. }| *name == asset_name)
        .with_context(|| format!("Could not find {asset_name:?} in {html_url}"))?;

    Ok(GhAsset {
        octocrab: octocrab.clone(),
        repo: repo.clone(),
        tag: tag_name,
        body,
        id,
        name,
        size: size as _,
    })
}

/// `find_gh_asset`に用いる。
///
/// 候補が複数あった場合、「デバイス」の数が最も小さいもののうち最初のものを選ぶ。
fn find_onnxruntime(
    body: &str, // リリースの"body" (i.e. リリースノートの内容)
    os: Os,
    cpu_arch: CpuArch,
    devices: &BTreeSet<Device>,
) -> anyhow::Result<String> {
    const TARGET: &str = "table\
        [data-voicevox-onnxruntime-specs-format-version=\"1\"]\
        [data-voicevox-onnxruntime-specs-type=\"dylibs\"]";

    html_blocks(body)
        .iter()
        .flat_map(|html_block| html_block.select(selector!(TARGET)))
        .exactly_one()
        .map_err(|err| match err.count() {
            0 => anyhow!("リリースノートの中に`{TARGET}`が見つかりませんでした"),
            _ => anyhow!("リリースノートの中に`{TARGET}`が複数ありました"),
        })?
        .select(selector!("tbody > tr"))
        .map(|tr| {
            tr.select(selector!("td"))
                .map(|td| td.text().exactly_one().ok())
                .collect::<Option<Vec<_>>>()
                .and_then(|text| text.try_into().ok())
                .with_context(|| format!("リリースノート中の`{TARGET}`をパースできませんでした"))
        })
        .collect::<Result<Vec<[_; _]>, _>>()?
        .into_iter()
        .filter(|&[spec_os, spec_cpu_arch, spec_devices, _]| {
            spec_os
                == match os {
                    Os::Windows => "Windows",
                    Os::Linux => "Linux",
                    Os::Osx => "macOS",
                    Os::Android => "Android",
                    Os::Ios => "iOS",
                }
                && spec_cpu_arch
                    == match cpu_arch {
                        CpuArch::X86 => "x86",
                        CpuArch::X64 => "x86_64",
                        CpuArch::Arm64 => "AArch64",
                    }
                && devices.iter().all(|device| {
                    spec_devices.split('/').any(|spec_device| {
                        spec_device
                            == match device {
                                Device::Cpu => "CPU",
                                Device::Cuda => "CUDA",
                                Device::Directml => "DirectML",
                            }
                    })
                })
        })
        .min_by_key(|&[.., spec_devices, _]| spec_devices.split('/').count())
        .map(|[.., name]| name.to_owned())
        .with_context(|| "指定されたOS, アーキテクチャ, デバイスを含むものが見つかりませんでした")
}

fn extract_voicevox_onnxruntime_terms(asset: &GhAsset) -> anyhow::Result<Option<String>> {
    const TARGET: &str = "pre[data-voicevox-onnxruntime-terms] > code";

    let GhAsset {
        body: Some(body), ..
    } = asset
    else {
        return Ok(None);
    };

    match &*html_blocks(body)
        .iter()
        .flat_map(|html_block| html_block.select(selector!(TARGET)))
        .collect::<Vec<_>>()
    {
        [] => Ok(None),
        [terms] => {
            let terms = terms
                .text()
                .exactly_one()
                .map_err(|e| anyhow!("should be exactly_one, got {n} fragments", n = e.count()))?
                .to_owned();
            Ok(Some(terms))
        }
        [..] => bail!("リリースノートの中に`{TARGET}`が複数ありました"),
    }
}

fn html_blocks(markdown: &str) -> Vec<scraper::Html> {
    comrak::parse_document(&Default::default(), markdown, &Default::default())
        .descendants()
        .flat_map(|node| match &node.data.borrow().value {
            comrak::nodes::NodeValue::HtmlBlock(comrak::nodes::NodeHtmlBlock {
                literal, ..
            }) => Some(scraper::Html::parse_fragment(literal)),
            _ => None,
        })
        .collect::<Vec<_>>()
}

macro_rules! selector {
    ($expr:expr $(,)?) => {{
        static SELECTOR: LazyLock<scraper::Selector> =
            LazyLock::new(|| scraper::Selector::parse($expr).expect("should be valid"));
        &SELECTOR
    }};
}
use selector;

/// ダウンロードすべきモデル、利用規約を見つける。その際ユーザーに利用規約の同意を求める。
async fn find_models(
    octocrab: &Arc<Octocrab>,
    repo: &RepoName,
    version: Option<&Version>,
    pattern: &glob::Pattern,
) -> anyhow::Result<ModelsWithTerms> {
    let repos = octocrab.repos(&repo.owner, &repo.repo);

    let Release {
        html_url,
        tag_name,
        assets,
        ..
    } = if let Some(version) = version {
        repos.releases().get_by_tag(&version.to_string()).await?
    } else {
        let (_, release) = repos
            .releases()
            .list()
            .per_page(100)
            .send()
            .await?
            .into_stream(octocrab)
            .try_filter(
                |&Release {
                     draft, prerelease, ..
                 }| future::ready(!(draft || prerelease)),
            )
            .map(|release| {
                let release = release?;
                let tag = release
                    .tag_name
                    .parse::<Version>()
                    .with_context(|| format!("`{repo}` contains non-SemVer tags"))?;
                anyhow::Ok((tag, release))
            })
            .try_filter(|(tag, _)| future::ready(SUPPORTED_MODELS_VERSIONS.matches(tag)))
            .try_collect::<Vec<_>>()
            .await?
            .into_iter()
            .max_by(|(tag1, _), (tag2, _)| tag1.cmp(tag2))
            .with_context(|| {
                format!(
                    "{repo}の`{SUPPORTED_MODELS_VERSIONS}`の範囲には、\
                     pre-releaseではないリリースがありません",
                    SUPPORTED_MODELS_VERSIONS = *SUPPORTED_MODELS_VERSIONS,
                )
            })?;
        release
    };

    let find_by_name = |name| {
        assets
            .iter()
            .find(|a| a.name == name)
            .with_context(|| anyhow!("could not find `{name}` in '{html_url}'"))
    };

    let terms = find_by_name(MODELS_TERMS_FILE)?;
    let terms = repos.fetch_asset_utf8_content(terms).await?;

    let readme = find_by_name(MODELS_README_FILENAME)?;
    let readme = repos.fetch_asset_utf8_content(readme).await?;

    let models = assets
        .into_iter()
        .filter(|Asset { name, .. }| {
            ![MODELS_README_FILENAME, MODELS_TERMS_FILE].contains(&&**name) && pattern.matches(name)
        })
        .map(|Asset { id, name, size, .. }| VvmAsset {
            octocrab: octocrab.clone(),
            repo: repo.clone(),
            id,
            name,
            size: size as _,
        })
        .collect();

    return Ok(ModelsWithTerms {
        tag: tag_name,
        readme,
        terms,
        models,
    });

    #[ext]
    impl RepoHandler<'_> {
        async fn fetch_asset_utf8_content(&self, asset: &Asset) -> anyhow::Result<String> {
            let content = self
                .release_assets()
                .stream(asset.id.0)
                .await?
                .try_fold(
                    Vec::with_capacity(asset.size as _),
                    |mut content, chunk| async move {
                        content.extend_from_slice(&chunk);
                        Ok(content)
                    },
                )
                .await?;
            let content = String::from_utf8(content)
                .with_context(|| format!("`{}` is not valid UTF-8", asset.name))?;
            validate_models_readme_or_terms_txt(content)
        }
    }
}

fn ensure_confirmation(terms: &IndexMap<&'static str, &str>) -> anyhow::Result<()> {
    if terms.is_empty() {
        return Ok(());
    }

    let terms_pretty = &{
        let Some(max_line_width) = terms
            .values()
            .flat_map(|terms| terms.lines())
            .map(|line| line.width())
            .max()
        else {
            return Ok(());
        };

        let mut terms_pretty = "\
            ダウンロードには以下の利用規約への同意が必要です。\n\
            （矢印キーで移動、q で終了）\n"
            .to_owned();
        terms_pretty += &format!("─┬─{}\n", "─".repeat(max_line_width));
        let mut it = terms.values().peekable();
        while let Some(terms) = it.next() {
            for line in terms.lines() {
                terms_pretty += &format!(" │ {line}\n");
            }
            let joint = if it.peek().is_some() { '┼' } else { '┴' };
            terms_pretty += &format!("─{joint}─{}\n", "─".repeat(max_line_width));
        }
        terms_pretty
    };

    return loop {
        // どうも非ASCII文字が全体的に駄目らしくパニックする場合があるので、
        // そのときはページングせずに表示する。

        let result = std::panic::catch_unwind(|| {
            minus::page_all({
                let pager = minus::Pager::new();
                pager.set_text(terms_pretty)?;
                pager.set_prompt(
                    "上下キーとスペースでスクロールし、読み終えたらqを押してください",
                )?;
                pager.set_exit_strategy(minus::ExitStrategy::PagerQuit)?;
                pager
            })
        });

        if let Ok(result) = result {
            result?;
        } else {
            error!("something went wrong with the pager");
            print!("{terms_pretty}");
            io::stdout().flush()?;
        }

        match ask(terms)? {
            UserInput::Yes => break Ok(()),
            UserInput::No => bail!("you must agree with the term of use"),
            UserInput::ReadAgain => {}
        }
    };

    fn ask(terms: &IndexMap<&'static str, impl Sized>) -> anyhow::Result<UserInput> {
        loop {
            let input = rprompt::prompt_reply_from_bufread(
                &mut io::stdin().lock(),
                &mut io::stderr(),
                format!(
                    "[Agreement Required]\n\
                     {terms}に同意しますか？\n\
                     同意する場合は y を、同意しない場合は n を、再確認する場合は r を入力し、\
                     エンターキーを押してください。\n\
                     [y,n,r] : ",
                    terms = terms
                        .keys()
                        .format_with("と", |terms, f| f(&format_args!("「{terms}」"))),
                ),
            )?;
            if ["y", "yes"].contains(&&*input.to_lowercase()) {
                break Ok(UserInput::Yes);
            }
            if ["n", "no"].contains(&&*input.to_lowercase()) {
                break Ok(UserInput::No);
            }
            if ["r"].contains(&&*input.to_lowercase()) {
                break Ok(UserInput::ReadAgain);
            }
            if !io::stdin().is_terminal() {
                bail!("the stdin is not a TTY but received invalid input: {input:?}");
            }
        }
    }

    enum UserInput {
        Yes,
        No,
        ReadAgain,
    }
}

fn download_and_extract_from_gh(
    GhAsset {
        octocrab,
        repo,
        id,
        name,
        size,
        ..
    }: GhAsset,
    stripping: Stripping,
    output: PathBuf,
    progresses: &MultiProgress,
    tries: Tries,
) -> anyhow::Result<impl Future<Output = anyhow::Result<()>> + use<>> {
    let archive_kind = ArchiveKind::from_filename(&name)?;
    let pb = add_progress_bar(progresses, size, name);

    Ok(retry(tries, async move || {
        let bytes_stream = octocrab
            .repos(&repo.owner, &repo.repo)
            .release_assets()
            .stream(id.0)
            .await?
            .map_err(Into::into);

        download_and_extract(
            bytes_stream,
            Some(size),
            archive_kind,
            stripping,
            WebService::Github,
            &output,
            pb.clone(),
        )
        .await
    }))
}

async fn download_models(
    ModelsWithTerms {
        readme,
        terms,
        models,
        ..
    }: ModelsWithTerms,
    output: PathBuf,
    progresses: &MultiProgress,
    tries: Tries,
) -> anyhow::Result<impl Future<Output = anyhow::Result<()>> + use<>> {
    let models = models
        .into_iter()
        .map(|model| {
            let pb = add_progress_bar(progresses, model.size, model.name.clone());
            (model, pb)
        })
        .collect::<Vec<_>>();

    fs_err::tokio::create_dir_all(output.join(MODELS_DIR_NAME)).await?;
    fs_err::tokio::write(output.join(MODELS_README_FILENAME), readme).await?;
    fs_err::tokio::write(output.join(MODELS_TERMS_FILE), terms).await?;
    Ok(retry(tries, async move || {
        models
            .iter()
            .map(|(a, b)| fetch_model(a, b, &output))
            .collect::<FuturesUnordered<_>>()
            .try_collect::<()>()
            .await
    }))
}

async fn fetch_model(
    VvmAsset {
        octocrab,
        repo,
        id,
        name,
        size,
    }: &VvmAsset,
    pb: &ProgressBar,
    output: &Path,
) -> anyhow::Result<()> {
    let bytes_stream = octocrab
        .repos(&repo.owner, &repo.repo)
        .release_assets()
        .stream(id.0)
        .await?
        .map_err(Into::into);
    let pb = with_style(pb.clone(), &PROGRESS_STYLE1).await?;
    let model = download(
        bytes_stream,
        Some(*size),
        FileKind::ZipOrVvm,
        WebService::Github,
        pb.clone(),
    )
    .await?;
    let pb = tokio::task::spawn_blocking(move || {
        pb.set_style(PROGRESS_STYLE2.clone());
        pb.set_message("Writing...");
        pb
    })
    .await?;
    fs_err::tokio::write(output.join(MODELS_DIR_NAME).join(name), model).await?;
    tokio::task::spawn_blocking(move || pb.finish_with_message("Done!")).await?;
    Ok(())
}

fn add_progress_bar(
    progresses: &MultiProgress,
    len: u64,
    prefix: impl Into<Cow<'static, str>>,
) -> ProgressBar {
    let pb = progresses.add(ProgressBar::new(len));
    pb.set_style(PROGRESS_STYLE0.clone());
    pb.enable_steady_tick(INTERVAL);
    pb.set_prefix(prefix);
    return pb;

    const INTERVAL: Duration = Duration::from_millis(100);
}

async fn download_and_extract(
    bytes_stream: impl Stream<Item = anyhow::Result<Bytes>> + Unpin,
    content_length: Option<u64>,
    archive_kind: ArchiveKind,
    stripping: Stripping,
    WebService::Github: WebService,
    output: &Path,
    pb: ProgressBar,
) -> anyhow::Result<()> {
    let pb = with_style(pb, &PROGRESS_STYLE1).await?;
    let archive = download(
        bytes_stream,
        content_length,
        archive_kind.into(),
        WebService::Github,
        pb.clone(),
    )
    .await?;

    let pb = with_style(pb, &PROGRESS_STYLE2).await?;
    let files = &read_archive(archive, archive_kind, pb.clone()).await?;
    return extract(files, stripping, output, pb).await;

    async fn read_archive(
        archive: Vec<u8>,
        archive_kind: ArchiveKind,
        pb: ProgressBar,
    ) -> anyhow::Result<Vec<(PathBuf, Vec<u8>)>> {
        tokio::task::spawn_blocking(move || pb.set_message("Inflating...")).await?;

        tokio::task::spawn_blocking(move || match archive_kind {
            ArchiveKind::Zip => read_zip(&archive),
            ArchiveKind::Tgz => read_tgz(&archive),
        })
        .await?
    }

    fn read_zip(zip: &[u8]) -> anyhow::Result<Vec<(PathBuf, Vec<u8>)>> {
        let zip = ZipArchive::new(Cursor::new(zip))?;

        (0..zip.len())
            .into_par_iter()
            .map(|i| {
                let mut zip = zip.clone();
                let entry = zip.by_index(i)?;
                if entry.is_dir() {
                    return Ok(None);
                }
                let filename = entry.mangled_name();
                let size = entry.size() as _;
                let content = read_bytes(entry, size)?;
                Ok(Some((filename, content)))
            })
            .flat_map(Result::transpose)
            .collect()
    }

    fn read_tgz(tgz: &[u8]) -> anyhow::Result<Vec<(PathBuf, Vec<u8>)>> {
        binstall_tar::Archive::new(GzDecoder::new(tgz))
            .entries()?
            .map(|entry| {
                let entry = entry?;
                if !entry.header().entry_type().is_file() {
                    return Ok(None);
                }
                let path = entry.path()?.into_owned();
                let size = entry.size() as _;
                let content = read_bytes(entry, size)?;
                Ok(Some((path, content)))
            })
            .flat_map(Result::transpose)
            .collect()
    }

    fn read_bytes(mut rdr: impl Read, size: usize) -> io::Result<Vec<u8>> {
        let mut buf = Vec::with_capacity(size);
        rdr.read_to_end(&mut buf)?;
        Ok(buf)
    }

    async fn extract(
        files: &[(PathBuf, Vec<u8>)],
        stripping: Stripping,
        output: &Path,
        pb: ProgressBar,
    ) -> anyhow::Result<()> {
        let pb = tokio::task::spawn_blocking(move || {
            pb.set_message("Writing files...");
            pb
        })
        .await?;

        for (filename, content) in files {
            let filename = filename
                .iter()
                .skip(match stripping {
                    Stripping::None => 0,
                    Stripping::FirstDir => 1,
                })
                .collect::<PathBuf>();
            let dst = &output.join(filename);
            if let Some(parent) = dst.parent() {
                fs_err::tokio::create_dir_all(parent).await?;
            }
            fs_err::tokio::write(dst, content).await?;
        }

        tokio::task::spawn_blocking(move || pb.finish_with_message("Done!")).await?;
        Ok(())
    }
}

async fn with_style(
    pb: ProgressBar,
    style: &'static ProgressStyle,
) -> Result<ProgressBar, JoinError> {
    tokio::task::spawn_blocking(move || {
        pb.set_style(style.clone());
        pb
    })
    .await
}

async fn download(
    mut bytes_stream: impl Stream<Item = anyhow::Result<Bytes>> + Unpin,
    content_length: Option<u64>,
    kind: FileKind,
    WebService::Github: WebService,
    pb: ProgressBar,
) -> anyhow::Result<Vec<u8>> {
    if let Some(content_length) = content_length {
        pb.set_length(content_length);
    }

    return with_progress(pb, async move |pos_tx| {
        let mut downloaded = Vec::with_capacity(content_length.unwrap_or(0) as _);
        while let Some(chunk) = bytes_stream.next().await.transpose()? {
            downloaded.extend_from_slice(&chunk);
            pos_tx.send(downloaded.len() as _)?;
        }
        validate_archive_file(downloaded, kind, WebService::Github)
    })
    .await;

    async fn with_progress<Fun, Fut, Out>(pb: ProgressBar, f: Fun) -> anyhow::Result<Out>
    where
        Fun: FnOnce(tokio::sync::mpsc::UnboundedSender<u64>) -> Fut,
        Fut: Future<Output = anyhow::Result<Out>>,
    {
        let (pos_tx, mut pos_rx) = tokio::sync::mpsc::unbounded_channel();

        let (result1, result2) = futures_util::future::join(
            tokio::task::spawn_blocking(move || {
                while let Some(pos) = pos_rx.blocking_recv() {
                    pb.set_position(pos);
                }
            }),
            f(pos_tx),
        )
        .await;

        result1?;
        result2
    }
}

// FIXME: HTTPステータスを確認したりSHA-256をチェックしたりといったまともな方法にする。
// ただしどちらもoctocrab側の対応が必要。
// cf. https://github.com/VOICEVOX/voicevox_core/issues/1120
// その際、`download_and_extract_from_sourceforge`の名前も`…_from_url`に戻す。
/// [`octocrab::repo::ReleaseAssetsHandler::stream`]でダウンロードしたものを検証する。
fn validate_archive_file(
    content: Vec<u8>,
    content_kind: FileKind,
    WebService::Github: WebService,
) -> anyhow::Result<Vec<u8>> {
    match (content_kind, &*content) {
        (FileKind::ZipOrVvm, [0x50, 0x4b, 0x03, 0x04, ..])
        | (FileKind::Tgz, [0x1f, 0x8b, 0x08, ..]) => Ok(content),
        (_, content) => Err(error_for_unexpected_file_content(
            content,
            WebService::Github,
        )),
    }
}

// FIXME: `validate_archive_file`と同様。
/// [`DownloadTarget::Models`]におけるreadmeと利用規約のテキストを検証する。
fn validate_models_readme_or_terms_txt(content: String) -> anyhow::Result<String> {
    if content.starts_with("# VOICEVOX ") {
        Ok(content)
    } else {
        Err(error_for_unexpected_file_content(
            content.as_ref(),
            WebService::Github,
        ))
    }
}

fn error_for_unexpected_file_content(
    content: &[u8],
    WebService::Github: WebService,
) -> anyhow::Error {
    let mut msg = format!("予期しない応答を{}が返しました", WebService::Github);
    if let Ok(content) = str::from_utf8(content) {
        msg += ": ";
        msg += content.trim_end();
        if content.contains("API rate limit exceeded for") {
            msg += " (Note: レートリミットによるエラーの可能性があります。\
                    認証トークンを設定すると制限が緩和されます。\
                    詳細は`--help`をご覧ください)";
        }
    }
    anyhow!("{msg}")
}

#[derive(Clone, Copy, Display)]
enum WebService {
    #[strum(to_string = "GitHub")]
    Github,
}

struct GhAsset {
    octocrab: Arc<Octocrab>,
    repo: RepoName,
    tag: String,
    body: Option<String>,
    id: AssetId,
    name: String,
    size: u64,
}

struct ModelsWithTerms {
    tag: String,
    readme: String,
    terms: String,
    models: Vec<VvmAsset>,
}

struct VvmAsset {
    octocrab: Arc<Octocrab>,
    repo: RepoName,
    id: AssetId,
    name: String,
    size: u64,
}

#[derive(Clone, Copy)]
enum FileKind {
    ZipOrVvm,
    Tgz,
}

impl From<ArchiveKind> for FileKind {
    fn from(archive_kind: ArchiveKind) -> Self {
        match archive_kind {
            ArchiveKind::Zip => Self::ZipOrVvm,
            ArchiveKind::Tgz => Self::Tgz,
        }
    }
}

#[derive(Clone, Copy)]
enum ArchiveKind {
    Zip,
    Tgz,
}

impl ArchiveKind {
    fn from_filename(filename: &str) -> anyhow::Result<Self> {
        if filename.ends_with(".zip") {
            Ok(Self::Zip)
        } else if filename.ends_with(".tar.gz") || filename.ends_with(".tgz") {
            Ok(Self::Tgz)
        } else {
            bail!("unsupported filetype: {filename}");
        }
    }
}

#[derive(Clone, Copy)]
enum Stripping {
    None,
    FirstDir,
}

#[cfg(test)]
mod tests {
    use clap::Parser as _;
    use rstest::rstest;

    use super::Args;

    #[rstest]
    #[case(&["", "--only", "c-api", "--exclude", "models"])]
    #[case(&["", "--min", "--only", "c-api"])]
    #[case(&["", "--min", "--exclude", "c-api"])]
    fn it_denies_conflicting_options(#[case] args: &[&str]) {
        let result = Args::try_parse_from(args).map(|_| ()).map_err(|e| e.kind());
        assert_eq!(Err(clap::error::ErrorKind::ArgumentConflict), result);
    }
}
