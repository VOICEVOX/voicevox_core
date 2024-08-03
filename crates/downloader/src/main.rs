use std::{
    borrow::Cow,
    collections::{BTreeSet, HashSet},
    env,
    future::Future,
    io::{self, Cursor, Read},
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use anyhow::{anyhow, bail, Context as _};
use bytes::Bytes;
use clap::{Parser as _, ValueEnum};
use flate2::read::GzDecoder;
use futures_core::Stream;
use futures_util::{stream::FuturesOrdered, StreamExt as _, TryStreamExt as _};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use itertools::Itertools as _;
use octocrab::{
    models::{
        repos::{Asset, Release},
        AssetId,
    },
    Octocrab,
};
use once_cell::sync::Lazy;
use rayon::iter::{IntoParallelIterator as _, ParallelIterator as _};
use strum::{Display, IntoStaticStr};
use tokio::task::{JoinError, JoinSet};
use tracing::{info, warn};
use url::Url;
use zip::ZipArchive;

const DEFAULT_OUTPUT: &str = if cfg!(windows) {
    r".\voicevox_core"
} else {
    "./voicevox_core"
};

const LIB_NAME: &str = "voicevox_core";
const DEFAULT_CORE_REPO: &str = "VOICEVOX/voicevox_core";
const DEFAULT_ONNXRUNTIME_BUILDER_REPO: &str = "VOICEVOX/onnxruntime-builder";
const DEFAULT_ADDITIONAL_LIBRARIES_REPO: &str = "VOICEVOX/voicevox_additional_libraries";

static OPEN_JTALK_DIC_URL: Lazy<Url> = Lazy::new(|| {
    "https://jaist.dl.sourceforge.net/project/open-jtalk/Dictionary/open_jtalk_dic-1.11/open_jtalk_dic_utf_8-1.11.tar.gz"
        .parse()
        .unwrap()
});

#[derive(clap::Parser)]
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

    /// `--only core`のエイリアス
    #[arg(long, conflicts_with("additional_libraries_version"))]
    min: bool,

    /// 出力先の指定
    #[arg(short, long, value_name("DIRECTORY"), default_value(DEFAULT_OUTPUT))]
    output: PathBuf,

    /// ダウンロードするvoicevox_coreのバージョンの指定
    #[arg(short, long, value_name("GIT_TAG_OR_LATEST"), default_value("latest"))]
    version: String,

    /// ダウンロードするONNX Runtimeのバージョンの指定
    #[arg(long, value_name("GIT_TAG_OR_LATEST"), default_value("latest"))]
    onnxruntime_version: String,

    /// 追加でダウンロードするライブラリのバージョン
    #[arg(long, value_name("GIT_TAG_OR_LATEST"), default_value("latest"))]
    additional_libraries_version: String,

    /// ダウンロードするデバイスを指定する(cudaはlinuxのみ)
    #[arg(value_enum, long, num_args(1..), default_value(<&str>::from(Device::default())))]
    devices: Vec<Device>,

    /// ダウンロードするcpuのアーキテクチャを指定する
    #[arg(value_enum, long, default_value(CpuArch::default_opt().map(<&str>::from)))]
    cpu_arch: CpuArch,

    /// ダウンロードする対象のOSを指定する
    #[arg(value_enum, long, default_value(Os::default_opt().map(<&str>::from)))]
    os: Os,

    #[arg(long, value_name("REPOSITORY"), default_value(DEFAULT_CORE_REPO))]
    core_repo: RepoName,

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
}

#[derive(ValueEnum, Clone, Copy, PartialEq, Eq, Hash)]
enum DownloadTarget {
    Core,
    Models,
    Onnxruntime,
    AdditionalLibraries,
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

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    setup_logger();

    let Args {
        only,
        exclude,
        min,
        output,
        version,
        onnxruntime_version,
        additional_libraries_version,
        devices,
        cpu_arch,
        os,
        core_repo,
        onnxruntime_builder_repo,
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
        [DownloadTarget::Core].into()
    } else {
        DownloadTarget::value_variants().iter().copied().collect()
    };

    if !(targets.contains(&DownloadTarget::Core) || targets.contains(&DownloadTarget::Models)) {
        if version != "latest" {
            warn!(
                "`--version={version}`が指定されていますが、`core`も`models`もダウンロード対象から\
                 除外されています",
            );
        }
        if core_repo.to_string() != DEFAULT_CORE_REPO {
            warn!(
                "`--core-repo={core_repo}`が指定されていますが、`core`も`models`もダウンロード対象\
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

    let octocrab = &octocrab()?;

    let core = find_gh_asset(octocrab, &core_repo, &version, |tag, _| {
        Ok(format!("{LIB_NAME}-{os}-{cpu_arch}-{tag}.zip"))
    })
    .await?;

    let model = find_gh_asset(octocrab, &core_repo, &version, |tag, _| {
        Ok(format!("model-{tag}.zip"))
    })
    .await?;

    let onnxruntime = find_gh_asset(
        octocrab,
        &onnxruntime_builder_repo,
        &onnxruntime_version,
        |_, body| {
            let body = body.with_context(|| "リリースノートがありません")?;
            find_onnxruntime(body, os, cpu_arch, &devices)
        },
    )
    .await?;

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
    info!("ダウンロード{LIB_NAME}バージョン: {}", core.tag);
    info!("ダウンロードONNX Runtimeバージョン: {}", onnxruntime.tag);
    if !additional_libraries.is_empty() {
        info!(
            "ダウンロード追加ライブラリバージョン: {}",
            additional_libraries
                .iter()
                .map(|GhAsset { tag, .. }| tag)
                .format(", "),
        );
    }

    let progresses = MultiProgress::new();

    let mut tasks = JoinSet::new();

    if targets.contains(&DownloadTarget::Core) {
        tasks.spawn(download_and_extract_from_gh(
            core,
            Stripping::FirstDir,
            &output,
            &progresses,
        )?);
    }
    if targets.contains(&DownloadTarget::Models) {
        tasks.spawn(download_and_extract_from_gh(
            model,
            Stripping::FirstDir,
            &output.join("model"),
            &progresses,
        )?);
    }
    if targets.contains(&DownloadTarget::Onnxruntime) {
        tasks.spawn(download_and_extract_from_gh(
            onnxruntime,
            Stripping::FirstDir,
            &output.join("onnxruntime"),
            &progresses,
        )?);
    }
    if targets.contains(&DownloadTarget::AdditionalLibraries) {
        for additional_libraries in additional_libraries {
            tasks.spawn(download_and_extract_from_gh(
                additional_libraries,
                Stripping::FirstDir,
                &output,
                &progresses,
            )?);
        }
    }
    if targets.contains(&DownloadTarget::Dict) {
        tasks.spawn(download_and_extract_from_url(
            &OPEN_JTALK_DIC_URL,
            Stripping::None,
            &output,
            &progresses,
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
            "`{repo}`の`{tag_name}`の中から条件に合致するビルドが見つけることができませんでした",
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
    macro_rules! selector {
        ($expr:expr $(,)?) => {{
            static SELECTOR: Lazy<scraper::Selector> =
                Lazy::new(|| scraper::Selector::parse($expr).expect("should be valid"));
            &SELECTOR
        }};
    }

    const TARGET: &str = "table\
        [data-voicevox-onnxruntime-specs-format-version=\"1\"]\
        [data-voicevox-onnxruntime-specs-type=\"dylibs\"]";

    comrak::parse_document(&Default::default(), body, &Default::default())
        .descendants()
        .flat_map(|node| match &node.data.borrow().value {
            comrak::nodes::NodeValue::HtmlBlock(comrak::nodes::NodeHtmlBlock {
                literal, ..
            }) => Some(scraper::Html::parse_fragment(literal)),
            _ => None,
        })
        .collect::<Vec<_>>()
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
    output: &Path,
    progresses: &MultiProgress,
) -> anyhow::Result<impl Future<Output = anyhow::Result<()>>> {
    let output = output.to_owned();
    let archive_kind = ArchiveKind::from_filename(&name)?;
    let pb = add_progress_bar(progresses, size as _, name);

    Ok(async move {
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
            pb,
        )
        .await
    })
}

fn download_and_extract_from_url(
    url: &'static Url,
    stripping: Stripping,
    output: &Path,
    progresses: &MultiProgress,
) -> anyhow::Result<impl Future<Output = anyhow::Result<()>>> {
    let output = output.to_owned();
    let name = url
        .path_segments()
        .and_then(|s| s.last())
        .unwrap_or_default();
    let archive_kind = ArchiveKind::from_filename(name)?;
    let pb = add_progress_bar(progresses, 0, name);

    Ok(async move {
        let res = reqwest::get(url.clone()).await?.error_for_status()?;
        let content_length = res.content_length();
        let bytes_stream = res.bytes_stream().map_err(Into::into);

        download_and_extract(
            bytes_stream,
            content_length,
            archive_kind,
            stripping,
            &output,
            pb,
        )
        .await
    })
}

fn add_progress_bar(
    progresses: &MultiProgress,
    len: u64,
    prefix: impl Into<Cow<'static, str>>,
) -> ProgressBar {
    let pb = progresses.add(ProgressBar::new(len));
    pb.set_style(PROGRESS_STYLE.clone());
    pb.enable_steady_tick(INTERVAL);
    pb.set_prefix(prefix);
    return pb;

    const INTERVAL: Duration = Duration::from_millis(100);

    static PROGRESS_STYLE: Lazy<ProgressStyle> =
        Lazy::new(|| ProgressStyle::with_template("{prefix}").unwrap());
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

    static PROGRESS_STYLE1: Lazy<ProgressStyle> = Lazy::new(|| {
        ProgressStyle::with_template(
            "{prefix:55} {bytes:>11} {bytes_per_sec:>13} {elapsed_precise} {bar} {percent:>3}%",
        )
        .unwrap()
    });

    static PROGRESS_STYLE2: Lazy<ProgressStyle> =
        Lazy::new(|| ProgressStyle::with_template("{prefix:55} {spinner} {msg}").unwrap());

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

        with_progress(pb, |pos_tx| async move {
            let mut downloaded = Vec::with_capacity(content_length.unwrap_or(0) as _);
            while let Some(chunk) = bytes_stream.next().await.transpose()? {
                downloaded.extend_from_slice(&chunk);
                pos_tx.send(downloaded.len() as _)?;
            }
            Ok(downloaded)
        })
        .await
    }

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

struct GhAsset {
    octocrab: Arc<Octocrab>,
    repo: RepoName,
    tag: String,
    id: AssetId,
    name: String,
    size: usize,
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
    #[case(&["", "--only", "core", "--exclude", "models"])]
    #[case(&["", "--min", "--only", "core"])]
    #[case(&["", "--min", "--exclude", "core"])]
    fn it_denies_conflicting_options(#[case] args: &[&str]) {
        let result = Args::try_parse_from(args).map(|_| ()).map_err(|e| e.kind());
        assert_eq!(Err(clap::error::ErrorKind::ArgumentConflict), result);
    }
}
