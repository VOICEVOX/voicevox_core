use std::{
    borrow::Cow,
    collections::{BTreeSet, HashSet},
    env,
    future::Future,
    io::{self, Cursor, IsTerminal as _, Read, Write as _},
    num::NonZero,
    path::{Path, PathBuf},
    str::FromStr,
    sync::{Arc, LazyLock},
    time::Duration,
};

use anyhow::{Context as _, anyhow, bail, ensure};
use base64::{Engine as _, prelude::BASE64_STANDARD};
use bytes::Bytes;
use clap::{Parser as _, ValueEnum};
use easy_ext::ext;
use flate2::read::GzDecoder;
use futures_core::Stream;
use futures_util::{
    StreamExt as _, TryStreamExt as _,
    future::OptionFuture,
    stream::{FuturesOrdered, FuturesUnordered},
};
use indexmap::IndexMap;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use itertools::Itertools as _;
use octocrab::{
    Octocrab,
    models::{
        AssetId,
        repos::{Asset, CommitObject, Content, Release, Tag},
    },
    repos::RepoHandler,
};
use rayon::iter::{IntoParallelIterator as _, ParallelIterator as _};
use reqwest::Client;
use semver::VersionReq;
use strum::{Display, IntoStaticStr};
use tokio::task::{JoinError, JoinSet};
use tracing::{error, info, warn};
use unicode_width::UnicodeWidthStr as _;
use url::Url;
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

static ALLOWED_MODELS_VERSIONS: LazyLock<VersionReq> =
    LazyLock::new(|| ">=0.1,<0.2".parse().unwrap());
const MODELS_README_FILENAME: &str = "README.md";
const MODELS_README_RENAME: &str = "README.txt";
const MODELS_DIR_NAME: &str = "vvms";
const MODELS_TERMS_NAME: &str = "VOICEVOX 音声モデル 利用規約";
const MODELS_TERMS_FILE: &str = "TERMS.txt";

static OPEN_JTALK_DIC_URL: LazyLock<Url> = LazyLock::new(|| {
    "https://jaist.dl.sourceforge.net/project/open-jtalk/Dictionary/open_jtalk_dic-1.11/open_jtalk_dic_utf_8-1.11.tar.gz"
        .parse()
        .unwrap()
});

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
#[command(version)]
struct Args {
    /// ダウンロード対象を限定する
    #[arg(
        long,
        num_args(1..),
        value_name("TARGET"),
        conflicts_with_all(["exclude", "min"]))
    ]
    only: Vec<DownloadTarget>,

    /// ダウンロード対象を除外する
    #[arg(long, num_args(1..), value_name("TARGET"), conflicts_with("min"))]
    exclude: Vec<DownloadTarget>,

    /// `--only c-api`のエイリアス
    #[arg(long, conflicts_with("additional_libraries_version"))]
    min: bool,

    /// 出力先の指定
    #[arg(short, long, value_name("DIRECTORY"), default_value(DEFAULT_OUTPUT))]
    output: PathBuf,

    /// ダウンロードするVOICEVOX CORE C APIのバージョンの指定
    #[arg(long, value_name("GIT_TAG_OR_LATEST"), default_value("latest"))]
    c_api_version: String,

    /// ダウンロードするONNX Runtimeのバージョンの指定
    #[arg(long, value_name("GIT_TAG_OR_LATEST"), default_value("latest"))]
    onnxruntime_version: String,

    /// 追加でダウンロードするライブラリのバージョン
    #[arg(long, value_name("GIT_TAG_OR_LATEST"), default_value("latest"))]
    additional_libraries_version: String,

    #[arg(long, value_name("GLOB"), default_value("*"))]
    models_pattern: glob::Pattern,

    /// ダウンロードするデバイスを指定する(cudaはlinuxのみ)
    #[arg(value_enum, long, num_args(1..), default_value(<&str>::from(Device::default())))]
    devices: Vec<Device>,

    /// ダウンロードするcpuのアーキテクチャを指定する
    #[arg(value_enum, long, default_value(CpuArch::default_opt().map(<&str>::from)))]
    cpu_arch: CpuArch,

    /// ダウンロードする対象のOSを指定する
    #[arg(value_enum, long, default_value(Os::default_opt().map(<&str>::from)))]
    os: Os,

    /// ダウンロードにおける試行回数。'0'か'inf'で無限にリトライ
    #[arg(short, long, value_name("NUMBER"), default_value("5"))]
    tries: Tries,

    #[arg(long, value_name("REPOSITORY"), default_value(DEFAULT_C_API_REPO))]
    c_api_repo: RepoName,

    #[arg(
        long,
        value_name("REPOSITORY"),
        default_value(DEFAULT_ONNXRUNTIME_BUILDER_REPO)
    )]
    onnxruntime_builder_repo: RepoName,

    #[arg(
        long,
        value_name("REPOSITORY"),
        default_value(DEFAULT_ADDITIONAL_LIBRARIES_REPO)
    )]
    additional_libraries_repo: RepoName,

    #[arg(long, value_name("REPOSITORY"), default_value(DEFAULT_MODELS_REPO))]
    models_repo: RepoName,
}

#[derive(ValueEnum, Clone, Copy, PartialEq, Eq, Hash)]
enum DownloadTarget {
    CApi,
    Onnxruntime,
    AdditionalLibraries,
    Models,
    Dict,
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
}

impl Os {
    fn default_opt() -> Option<Self> {
        match env::consts::OS {
            "windows" => Some(Self::Windows),
            "linux" => Some(Self::Linux),
            "macos" => Some(Self::Osx),
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
    if !targets.contains(&DownloadTarget::Models) && models_pattern.as_str() != "*" {
        warn!(
            "`--models-pattern={models_pattern}`が指定されていますが、`models`はダウンロード対象\
             から除外されています",
            models_pattern = models_pattern.as_str(),
        );
    }

    let octocrab = &octocrab()?;

    let c_api = OptionFuture::from(targets.contains(&DownloadTarget::CApi).then(|| {
        find_gh_asset(octocrab, &c_api_repo, &c_api_version, |tag, _| {
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

    let models = OptionFuture::from(
        targets
            .contains(&DownloadTarget::Models)
            .then(|| find_models(octocrab, &models_repo, &models_pattern)),
    )
    .await
    .transpose()?;

    ensure_confirmation(
        &itertools::chain(
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

    let progresses = MultiProgress::new();

    let mut tasks = JoinSet::new();

    if let Some(c_api) = c_api {
        tasks.spawn(download_and_extract_from_gh(
            c_api,
            Stripping::FirstDir,
            output.join("c_api"),
            &progresses,
            tries,
        )?);
    }
    if let Some(onnxruntime) = onnxruntime {
        tasks.spawn(download_and_extract_from_gh(
            onnxruntime,
            Stripping::FirstDir,
            output.join("onnxruntime"),
            &progresses,
            tries,
        )?);
    }
    if targets.contains(&DownloadTarget::AdditionalLibraries) {
        for additional_libraries in additional_libraries {
            tasks.spawn(download_and_extract_from_gh(
                additional_libraries,
                Stripping::FirstDir,
                output.join("additional_libraries"),
                &progresses,
                tries,
            )?);
        }
    }
    if let Some(models) = models {
        tasks.spawn(download_models(models, output.join("models"), &progresses, tries).await?);
    }
    if targets.contains(&DownloadTarget::Dict) {
        tasks.spawn(download_and_extract_from_url(
            &OPEN_JTALK_DIC_URL,
            Stripping::None,
            output.join("dict"),
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

    // パーソナルトークン無しだと、GitHubのREST APIの利用に強い回数制限がかかる。
    // そのためCI上では`${{ secrets.GITHUB_TOKEN }}`を使わないとかなりの確率で失敗するようになる。
    // 手元の手動実行であってもやりすぎると制限に引っ掛かるので、手元でも`$GITHUB_TOKEN`を
    // 与えられるようにする。
    if let Ok(github_token) = env::var("GITHUB_TOKEN") {
        octocrab = octocrab.personal_token(github_token);
    }

    octocrab.build().map(Arc::new)
}

async fn retry<E, F>(tries: Tries, mut f: F) -> Result<(), E>
where
    F: AsyncFnMut() -> Result<(), E>,
{
    match tries {
        Tries::Infinite => loop {
            if let Ok(o) = f().await {
                return Ok(o);
            }
        },
        Tries::Finite(nonzero) => {
            for _ in 0..nonzero.get() - 1 {
                if let Ok(o) = f().await {
                    return Ok(o);
                }
            }
            f().await
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
        .collect::<Result<Vec<[_; 4]>, _>>()?
        .into_iter()
        .filter(|&[spec_os, spec_cpu_arch, spec_devices, _]| {
            spec_os
                == match os {
                    Os::Windows => "Windows",
                    Os::Linux => "Linux",
                    Os::Osx => "macOS",
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
    octocrab: &Octocrab,
    repo: &RepoName,
    pattern: &glob::Pattern,
) -> anyhow::Result<ModelsWithTerms> {
    let repos = octocrab.repos(&repo.owner, &repo.repo);

    let (tag, sha) = repos
        .list_tags()
        .send()
        .await?
        .into_iter()
        .map(
            |Tag {
                 name,
                 commit: CommitObject { sha, .. },
                 ..
             }| {
                let tag = name
                    .parse()
                    .with_context(|| format!("`{repo}` contains non-SemVer tags"))?;
                Ok((tag, sha))
            },
        )
        .collect::<anyhow::Result<Vec<_>>>()?
        .into_iter()
        .filter(|(version, _)| ALLOWED_MODELS_VERSIONS.matches(version))
        .sorted()
        .next_back()
        .with_context(|| format!("`{repo}`"))?;
    let tag = tag.to_string();

    let terms = repos.fetch_file_content(&sha, MODELS_TERMS_FILE).await?;
    let readme = repos
        .fetch_file_content(&sha, MODELS_README_FILENAME)
        .await?;

    let models = repos
        .get_content()
        .r#ref(&sha)
        .path(MODELS_DIR_NAME)
        .send()
        .await?
        .items
        .into_iter()
        .map(
            |Content {
                 name,
                 size,
                 download_url,
                 r#type,
                 ..
             }| {
                ensure!(r#type == "file", "found directory");
                if !pattern.matches(&name) {
                    return Ok(None);
                }
                Ok(Some(GhContent {
                    name,
                    download_url: download_url.expect("should present"),
                    size: size as _,
                }))
            },
        )
        .flat_map(Result::transpose)
        .collect::<anyhow::Result<_>>()?;

    return Ok(ModelsWithTerms {
        tag,
        readme,
        terms,
        models,
    });

    #[ext]
    impl RepoHandler<'_> {
        async fn fetch_file_content(&self, sha: &str, path: &str) -> anyhow::Result<String> {
            let Content {
                encoding, content, ..
            } = self
                .get_content()
                .r#ref(sha)
                .path(path)
                .send()
                .await?
                .items
                .into_iter()
                .exactly_one()
                .map_err(|_| anyhow!("could not find `{path}`"))?;

            ensure!(
                encoding.as_deref() == Some("base64"),
                r#"expected `encoding="base64"`"#,
            );

            let content = content.expect("should present").replace('\n', "");
            let content = BASE64_STANDARD.decode(content)?;
            let content = String::from_utf8(content)
                .with_context(|| format!("`{path}` is not valid UTF-8"))?;
            Ok(content)
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
    let pb = add_progress_bar(progresses, size as _, name);

    Ok(retry(tries, async move || {
        let bytes_stream = octocrab
            .repos(&repo.owner, &repo.repo)
            .releases()
            .stream_asset(id)
            .await?
            .map_err(Into::into);

        download_and_extract(
            bytes_stream,
            Some(size as _),
            archive_kind,
            stripping,
            &output,
            pb.clone(),
        )
        .await
    }))
}

fn download_and_extract_from_url(
    url: &'static Url,
    stripping: Stripping,
    output: PathBuf,
    progresses: &MultiProgress,
    tries: Tries,
) -> anyhow::Result<impl Future<Output = anyhow::Result<()>> + use<>> {
    let name = url
        .path_segments()
        .and_then(|s| { s }.next_back())
        .unwrap_or_default();
    let archive_kind = ArchiveKind::from_filename(name)?;
    let pb = add_progress_bar(progresses, 0, name);

    Ok(retry(tries, async move || {
        let res = reqwest::get(url.clone()).await?.error_for_status()?;
        let content_length = res.content_length();
        let bytes_stream = res.bytes_stream().map_err(Into::into);

        download_and_extract(
            bytes_stream,
            content_length,
            archive_kind,
            stripping,
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
    let reqwest = reqwest::Client::builder().build()?;

    let models = models
        .into_iter()
        .map(|model| {
            let pb = add_progress_bar(progresses, model.size as _, model.name.clone());
            (model, pb)
        })
        .collect::<Vec<_>>();

    fs_err::tokio::create_dir_all(output.join(MODELS_DIR_NAME)).await?;
    fs_err::tokio::write(output.join(MODELS_README_RENAME), readme).await?;
    fs_err::tokio::write(output.join(MODELS_TERMS_FILE), terms).await?;
    Ok(retry(tries, async move || {
        models
            .clone()
            .into_iter()
            .map(|(c, b)| fetch_model(c, b, reqwest.clone(), &output))
            .collect::<FuturesUnordered<_>>()
            .try_collect::<()>()
            .await
    }))
}

async fn fetch_model(
    content: GhContent,
    pb: ProgressBar,
    reqwest: Client,
    output: &Path,
) -> anyhow::Result<()> {
    let GhContent {
        name,
        download_url,
        size,
    } = content;
    let res = reqwest.get(download_url).send().await?.error_for_status()?;
    let bytes_stream = res.bytes_stream().map_err(Into::into);
    let pb = with_style(pb.clone(), &PROGRESS_STYLE1).await?;
    let model = download(bytes_stream, Some(size), pb.clone()).await?;
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
    output: &Path,
    pb: ProgressBar,
) -> anyhow::Result<()> {
    let pb = with_style(pb, &PROGRESS_STYLE1).await?;
    let archive = download(bytes_stream, content_length, pb.clone()).await?;

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
        Ok(downloaded)
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

#[derive(Clone)]
struct GhAsset {
    octocrab: Arc<Octocrab>,
    repo: RepoName,
    tag: String,
    body: Option<String>,
    id: AssetId,
    name: String,
    size: usize,
}

#[derive(Clone)]
struct ModelsWithTerms {
    tag: String,
    readme: String,
    terms: String,
    models: Vec<GhContent>,
}

#[derive(Clone)]
struct GhContent {
    name: String,
    download_url: String,
    size: u64,
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
