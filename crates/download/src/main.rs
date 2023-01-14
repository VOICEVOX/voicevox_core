use std::{
    env,
    io::{self, Cursor, Read},
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Context as _;
use bytes::Bytes;
use clap::{Parser as _, ValueEnum};
use flate2::read::GzDecoder;
use futures_core::Stream;
use futures_util::{StreamExt as _, TryFutureExt as _, TryStreamExt as _};
use indicatif::{MultiProgress, ProgressBar};
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
use tracing::info;
use url::Url;
use zip::ZipArchive;

const DEFAULT_OUTPUT: &str = if cfg!(windows) {
    r".\voicevox_core"
} else {
    "./voicevox_core"
};

const ORGANIZATION_NAME: &str = "VOICEVOX";
const CORE_REPO_NAME: &str = "voicevox_core";
const ADDITIONAL_LIBRARIES_REPO_NAME: &str = "voicevox_additional_libraries";

static OPEN_JTALK_DIC_URL: Lazy<Url> = Lazy::new(|| {
    "https://jaist.dl.sourceforge.net/project/open-jtalk/Dictionary/open_jtalk_dic-1.11/open_jtalk_dic_utf_8-1.11.tar.gz"
        .parse()
        .unwrap()
});

#[derive(clap::Parser)]
struct Args {
    /// ダウンロードするライブラリを最小限にするように指定
    #[arg(long)]
    min: bool,

    /// 出力先の指定
    #[arg(short, long, default_value(DEFAULT_OUTPUT))]
    output: PathBuf,

    /// ダウンロードするvoicevox_coreのバージョンの指定
    #[arg(short, long, default_value("latest"))]
    version: String,

    /// 追加でダウンロードするライブラリのバージョン
    #[arg(long, default_value("latest"))]
    additional_libraries_version: String,

    /// ダウンロードするacceleratorを指定する(cudaはlinuxのみ)
    #[arg(value_enum, long, default_value(<&str>::from(Accelerator::default())))]
    accelerator: Accelerator,

    /// ダウンロードするcpuのアーキテクチャを指定する
    #[arg(value_enum, long, default_value(CpuArch::default_opt().map(<&str>::from)))]
    cpu_arch: CpuArch,

    /// ダウンロードする対象のOSを指定する
    #[arg(value_enum, long, default_value(Os::default_opt().map(<&str>::from)))]
    os: Os,
}

#[derive(Default, ValueEnum, Display, IntoStaticStr, Clone, Copy, PartialEq)]
#[strum(serialize_all = "kebab-case")]
enum Accelerator {
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
    Aarch64,
}

impl CpuArch {
    fn default_opt() -> Option<Self> {
        match env::consts::ARCH {
            "x86_64" => Some(Self::X64),
            "aarch64" => Some(Self::Aarch64),
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

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    setup_logger();

    let Args {
        min,
        output,
        version,
        additional_libraries_version,
        accelerator,
        cpu_arch,
        os,
    } = Args::parse();

    let octocrab = &octocrab()?;

    let core = find_gh_asset(octocrab, CORE_REPO_NAME, &version, |tag| {
        let cpu_arch = match (os, cpu_arch) {
            (Os::Linux, CpuArch::Aarch64) => "arm64",
            (_, cpu_arch) => cpu_arch.into(),
        };
        let accelerator = match (os, accelerator) {
            (Os::Linux, Accelerator::Cuda) => "gpu",
            (_, accelerator) => accelerator.into(),
        };
        format!("{CORE_REPO_NAME}-{os}-{cpu_arch}-{accelerator}-{tag}.zip")
    })
    .await?;

    use futures_util::future::OptionFuture;

    let additional_libraries = OptionFuture::from((accelerator != Accelerator::Cpu).then(|| {
        find_gh_asset(
            octocrab,
            ADDITIONAL_LIBRARIES_REPO_NAME,
            &additional_libraries_version,
            |_| {
                let accelerator = match accelerator {
                    Accelerator::Cpu => unreachable!(),
                    Accelerator::Cuda => "CUDA",
                    Accelerator::Directml => "DirectML",
                };
                format!("{accelerator}-{os}-{cpu_arch}.zip")
            },
        )
    }))
    .await
    .transpose()?;

    info!("対象OS: {os}");
    info!("対象CPUアーキテクチャ: {cpu_arch}");
    info!("ダウンロードアーティファクトタイプ: {accelerator}");
    info!("ダウンロード{CORE_REPO_NAME}バージョン: {}", core.tag);
    if let Some(GhAsset { tag, .. }) = &additional_libraries {
        info!("ダウンロード追加ライブラリバージョン: {tag}");
    }

    let progresses = MultiProgress::new();

    let mut targets = vec![(
        Download::gh(core, &progresses),
        ArchiveKind::Zip,
        Stripping::FirstDir,
    )];

    if !min {
        if let Some(additional_libraries) = additional_libraries {
            targets.push((
                Download::gh(additional_libraries, &progresses),
                ArchiveKind::Zip,
                Stripping::FirstDir,
            ));
        }
        targets.push((
            Download::url(&OPEN_JTALK_DIC_URL, &progresses),
            ArchiveKind::Tgz,
            Stripping::None,
        ));
    }

    let archives = futures_util::future::join_all(
        targets
            .into_iter()
            .map(|(d, k, s)| download(d).map_ok(move |r| (r, k, s))),
    )
    .await
    .into_iter()
    .collect::<Result<Vec<_>, _>>()?;

    for (archive, kind, stripping) in archives {
        extract(&archive, kind, stripping, &output).await?;
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
    if let Ok(github_token) = env::var("GITHUB_TOKEN") {
        octocrab = octocrab.personal_token(github_token);
    }
    octocrab.build().map(Arc::new)
}

async fn find_gh_asset(
    octocrab: &Arc<Octocrab>,
    repo: &str,
    git_tag_or_latest: &str,
    asset_name: impl FnOnce(&str) -> String,
) -> anyhow::Result<GhAsset> {
    let Release {
        html_url,
        tag_name,
        assets,
        ..
    } = {
        let repos = octocrab.repos(ORGANIZATION_NAME, repo);
        let releases = repos.releases();
        match git_tag_or_latest {
            "latest" => releases.get_latest().await,
            tag => releases.get_by_tag(tag).await,
        }?
    };

    let asset_name = asset_name(&tag_name);
    let Asset { id, name, size, .. } = assets
        .into_iter()
        .find(|Asset { name, .. }| *name == asset_name)
        .with_context(|| format!("Could not find {asset_name:?} in {html_url}"))?;

    Ok(GhAsset {
        octocrab: octocrab.clone(),
        repo: repo.to_owned(),
        tag: tag_name,
        id,
        name,
        size: size as _,
    })
}

async fn download(Download { target, pb }: Download) -> anyhow::Result<Vec<u8>> {
    return match target {
        DownloadTarget::Gh(asset) => download_from_gh(&asset, &pb).await,
        DownloadTarget::Url(url) => download_from_url(url, &pb).await,
    };

    async fn download_from_gh(
        GhAsset {
            octocrab, repo, id, ..
        }: &GhAsset,
        pb: &ProgressBar,
    ) -> anyhow::Result<Vec<u8>> {
        let bytes_stream = octocrab
            .repos(ORGANIZATION_NAME, repo)
            .releases()
            .stream_asset(*id)
            .await?
            .map_err(Into::into);

        download(bytes_stream, None, pb).await
    }

    async fn download_from_url(url: &Url, pb: &ProgressBar) -> anyhow::Result<Vec<u8>> {
        let res = reqwest::get(url.clone()).await?.error_for_status()?;
        let content_length = res.content_length();
        let bytes_stream = res.bytes_stream().map_err(Into::into);

        download(bytes_stream, content_length, pb).await
    }

    async fn download(
        mut bytes_stream: impl Stream<Item = anyhow::Result<Bytes>> + Unpin,
        content_length: Option<u64>,
        pb: &ProgressBar,
    ) -> anyhow::Result<Vec<u8>> {
        if let Some(content_length) = content_length {
            pb.set_length(content_length);
        }

        let mut downloaded = vec![];

        while let Some(chunk) = bytes_stream.next().await.transpose()? {
            downloaded.extend_from_slice(&chunk);
            pb.set_position(downloaded.len() as _);
        }
        pb.finish();

        Ok(downloaded)
    }
}

async fn extract(
    archive: &[u8],
    kind: ArchiveKind,
    stripping: Stripping,
    output: &Path,
) -> anyhow::Result<()> {
    let files = match kind {
        ArchiveKind::Zip => read_zip(archive),
        ArchiveKind::Tgz => read_tgz(archive),
    }?;

    for (filename, content) in files {
        let filename = &match stripping {
            Stripping::FirstDir => filename.iter().skip(1).collect(),
            Stripping::None => filename,
        };
        let dst = &output.join(filename);
        if let Some(parent) = dst.parent() {
            fs_err::tokio::create_dir_all(parent).await?;
        }
        fs_err::tokio::write(dst, content).await?;
        info!("Wrote `{}`", dst.display());
    }
    return Ok(());

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
                let content = read_bytes(entry)?;
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
                let content = read_bytes(entry)?;
                Ok(Some((path, content)))
            })
            .flat_map(Result::transpose)
            .collect()
    }

    fn read_bytes(mut rdr: impl Read) -> io::Result<Vec<u8>> {
        let mut buf = vec![];
        rdr.read_to_end(&mut buf)?;
        Ok(buf)
    }

    fn strip_first_dir(posix_path: &str) -> &str {
        posix_path
            .find('/')
            .map(|ofs| &posix_path[ofs + 1..])
            .unwrap_or(posix_path)
    }
}

struct GhAsset {
    octocrab: Arc<Octocrab>,
    repo: String,
    tag: String,
    id: AssetId,
    name: String,
    size: usize,
}

struct Download {
    target: DownloadTarget,
    pb: ProgressBar,
}

impl Download {
    fn gh(asset: GhAsset, progresses: &MultiProgress) -> Self {
        let pb = progresses.add(ProgressBar::new(asset.size as _));
        pb.set_prefix(asset.name.clone());
        let target = DownloadTarget::Gh(asset);
        Self { target, pb }
    }

    fn url(url: &'static Url, progresses: &MultiProgress) -> Self {
        let pb = progresses.add(ProgressBar::new(0));
        pb.set_prefix(
            url.path_segments()
                .and_then(|s| s.last())
                .unwrap_or_default(),
        );
        let target = DownloadTarget::Url(url);
        Self { target, pb }
    }
}

enum DownloadTarget {
    Gh(GhAsset),
    Url(&'static Url),
}

#[derive(Clone, Copy)]
enum ArchiveKind {
    Zip,
    Tgz,
}

#[derive(Clone, Copy)]
enum Stripping {
    None,
    FirstDir,
}
