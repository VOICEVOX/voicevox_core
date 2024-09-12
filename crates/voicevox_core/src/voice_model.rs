//! 音声モデル（ VVM ファイル）。
//!
//! VVM ファイルの定義と形式は[ドキュメント](../../../docs/vvm.md)を参照。

use std::{
    marker::PhantomData,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{anyhow, Context as _};
use derive_more::From;
use easy_ext::ext;
use enum_map::{enum_map, EnumMap};
use futures_io::{AsyncBufRead, AsyncSeek};
use futures_util::future::{OptionFuture, TryFutureExt as _};
use itertools::Itertools as _;
use ouroboros::self_referencing;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    asyncs::Async,
    error::{LoadModelError, LoadModelErrorKind, LoadModelResult},
    infer::{
        domains::{InferenceDomainMap, TalkDomain, TalkOperation},
        InferenceDomain,
    },
    manifest::{Manifest, ManifestDomains, StyleIdToInnerVoiceId, TalkManifest},
    SpeakerMeta, StyleMeta, StyleType, VoiceModelMeta,
};

/// [`VoiceModelId`]の実体。
///
/// [`VoiceModelId`]: VoiceModelId
pub type RawVoiceModelId = Uuid;

pub(crate) type ModelBytesWithInnerVoiceIdsByDomain =
    (Option<(StyleIdToInnerVoiceId, EnumMap<TalkOperation, Vec<u8>>)>,);

/// 音声モデルID。
#[derive(
    PartialEq,
    Eq,
    Clone,
    Copy,
    Ord,
    Hash,
    PartialOrd,
    Deserialize,
    derive_more::Display,
    Debug,
    From,
)]
pub struct VoiceModelId(RawVoiceModelId);

impl VoiceModelId {
    pub fn raw_voice_model_id(self) -> RawVoiceModelId {
        self.0
    }
}

#[self_referencing]
struct Inner<A> {
    header: VoiceModelHeader,

    #[borrows(header)]
    #[not_covariant]
    inference_model_entries: InferenceDomainMap<InferenceModelEntries<'this>>,

    // `_marker`とすると、`borrow__marker`のような名前のメソッドが生成されて`non_snake_case`が
    // 起動してしまう
    marker: PhantomData<fn(A) -> A>,
}

impl<A: Async> Inner<A> {
    async fn from_path(path: impl AsRef<Path>) -> crate::Result<Self> {
        const MANIFEST_FILENAME: &str = "manifest.json";

        let path = path.as_ref();

        let error = |context, source| LoadModelError {
            path: path.to_owned(),
            context,
            source: Some(source),
        };

        let mut zip = A::open_zip(path)
            .await
            .map_err(|source| error(LoadModelErrorKind::OpenZipFile, source))?;

        let manifest = &async {
            let idx = zip.find_entry_index(MANIFEST_FILENAME)?;
            zip.read_file(idx).await
        }
        .await
        .map_err(|source| {
            error(
                LoadModelErrorKind::ReadZipEntry {
                    filename: MANIFEST_FILENAME.to_owned(),
                },
                source,
            )
        })?;
        let manifest = serde_json::from_slice::<Manifest>(manifest)
            .map_err(|source| error(LoadModelErrorKind::InvalidModelFormat, source.into()))?;

        let metas = &async {
            let idx = zip.find_entry_index(manifest.metas_filename())?;
            zip.read_file(idx).await
        }
        .await
        .map_err(|source| {
            error(
                LoadModelErrorKind::ReadZipEntry {
                    filename: manifest.metas_filename().clone(),
                },
                source,
            )
        })?;

        let header = VoiceModelHeader::new(manifest, metas, path)?;

        InnerTryBuilder {
            header,
            inference_model_entries_builder: |VoiceModelHeader { manifest, .. }| {
                manifest
                    .domains()
                    .each_ref()
                    .map(InferenceDomainMap {
                        talk: |talk| {
                            talk.as_ref()
                                .map(|manifest| {
                                    let indices = enum_map! {
                                        TalkOperation::PredictDuration => {
                                            zip.find_entry_index(&manifest.predict_duration_filename)?
                                        }
                                        TalkOperation::PredictIntonation => zip.find_entry_index(
                                            &manifest.predict_intonation_filename,
                                        )?,
                                        TalkOperation::Decode => {
                                            zip.find_entry_index(&manifest.decode_filename)?
                                        }
                                    };

                                    Ok(InferenceModelEntry { indices, manifest })
                                })
                                .transpose()
                                .map_err(move |source| {
                                    error(
                                        LoadModelErrorKind::ReadZipEntry {
                                            filename: MANIFEST_FILENAME.to_owned(),
                                        },
                                        source,
                                    )
                                })
                        },
                    })
                    .collect()
                    .map_err(crate::Error::from)
            },
            marker: PhantomData,
        }
        .try_build()
    }

    fn id(&self) -> VoiceModelId {
        self.borrow_header().manifest.id
    }

    fn metas(&self) -> &VoiceModelMeta {
        &self.borrow_header().metas
    }

    fn header(&self) -> &VoiceModelHeader {
        self.borrow_header()
    }

    async fn read_inference_models(
        &self,
    ) -> LoadModelResult<InferenceDomainMap<ModelBytesWithInnerVoiceIdsByDomain>> {
        let path = &self.borrow_header().path;

        let error = |context, source| LoadModelError {
            path: path.to_owned(),
            context,
            source: Some(source),
        };

        let mut zip = A::open_zip(path)
            .await
            .map_err(|source| error(LoadModelErrorKind::OpenZipFile, source))?;

        macro_rules! read_file {
            ($entry:expr $(,)?) => {{
                let (index, filename): (usize, Arc<str>) = $entry;
                zip.read_file(index)
                    .map_err(move |source| {
                        error(
                            LoadModelErrorKind::ReadZipEntry {
                                filename: (*filename).to_owned(),
                            },
                            source,
                        )
                    })
                    .await?
            }};
        }

        let InferenceDomainMap { talk } =
            self.with_inference_model_entries(|inference_model_entries| {
                inference_model_entries.each_ref().map(InferenceDomainMap {
                    talk: |talk| {
                        talk.as_ref()
                            .map(|InferenceModelEntry { indices, manifest }| {
                                (
                                    indices.map(|op, i| (i, manifest[op].clone())),
                                    manifest.style_id_to_inner_voice_id.clone(),
                                )
                            })
                    },
                })
            });

        let talk = OptionFuture::from(talk.map(
            |(entries, style_id_to_inner_voice_id)| async move {
                let [predict_duration, predict_intonation, decode] = entries.into_array();

                let predict_duration = read_file!(predict_duration);
                let predict_intonation = read_file!(predict_intonation);
                let decode = read_file!(decode);

                let model_bytes =
                    EnumMap::from_array([predict_duration, predict_intonation, decode]);

                Ok((style_id_to_inner_voice_id, model_bytes))
            },
        ))
        .await
        .transpose()?;

        Ok(InferenceDomainMap { talk })
    }
}

type InferenceModelEntries<'manifest> =
    (Option<InferenceModelEntry<TalkDomain, &'manifest TalkManifest>>,);

struct InferenceModelEntry<D: InferenceDomain, M> {
    indices: EnumMap<D::Operation, usize>,
    manifest: M,
}

#[ext]
impl<A: Async> A {
    async fn open_zip(
        path: &Path,
    ) -> anyhow::Result<async_zip::base::read::seek::ZipFileReader<impl AsyncBufRead + AsyncSeek>>
    {
        let zip = Self::open_file(path).await.with_context(|| {
            // fs-errのと同じにする
            format!("failed to open file `{}`", path.display())
        })?;
        let zip = futures_util::io::BufReader::new(zip); // async_zip v0.0.16では不要、v0.0.17では必要
        let zip = async_zip::base::read::seek::ZipFileReader::new(zip).await?;
        Ok(zip)
    }
}

#[ext]
impl<R: AsyncBufRead + AsyncSeek + Unpin> async_zip::base::read::seek::ZipFileReader<R> {
    fn find_entry_index(&self, filename: &str) -> anyhow::Result<usize> {
        let (idx, _) = self
            .file()
            .entries()
            .iter()
            .enumerate()
            .find(|(_, e)| e.filename().as_str().ok() == Some(filename))
            .with_context(|| "could not find `{filename}`")?;
        Ok(idx)
    }

    async fn read_file(&mut self, index: usize) -> anyhow::Result<Vec<u8>> {
        let mut rdr = self.reader_with_entry(index).await?;
        let mut buf = Vec::with_capacity(rdr.entry().uncompressed_size() as usize);
        rdr.read_to_end_checked(&mut buf).await?;
        Ok(buf)
    }
}

// FIXME: "header"といいつつ、VVMのファイルパスを持っている状態になっている。
/// 音声モデルが持つ、各モデルファイルの実体を除く情報。
///
/// モデルの`[u8]`と分けて`Status`に渡す。
pub(crate) struct VoiceModelHeader {
    pub(crate) manifest: Manifest,
    /// メタ情報。
    ///
    /// `manifest`が対応していない`StyleType`のスタイルは含まれるべきではない。
    pub(crate) metas: VoiceModelMeta,
    pub(crate) path: PathBuf,
}

impl VoiceModelHeader {
    fn new(manifest: Manifest, metas: &[u8], path: &Path) -> LoadModelResult<Self> {
        let error = |context, source| LoadModelError {
            path: path.to_owned(),
            context,
            source: Some(source),
        };

        let metas = serde_json::from_slice::<VoiceModelMeta>(metas).map_err(|source| {
            error(
                LoadModelErrorKind::InvalidModelFormat,
                anyhow::Error::from(source)
                    .context(format!("{}が不正です", manifest.metas_filename())),
            )
        })?;

        manifest
            .domains()
            .check_acceptable(&metas)
            .map_err(|style_type| {
                error(
                    LoadModelErrorKind::InvalidModelFormat,
                    anyhow!(
                        "{metas_filename}には`{style_type}`のスタイルが存在しますが、manifest.json\
                         での対応がありません",
                        metas_filename = manifest.metas_filename(),
                    ),
                )
            })?;

        Ok(Self {
            manifest,
            metas,
            path: path.to_owned(),
        })
    }
}

impl InferenceDomainMap<ManifestDomains> {
    /// manifestとして対応していない`StyleType`に対してエラーを発する。
    ///
    /// `Status`はこのバリデーションを信頼し、`InferenceDomain`の不足時にパニックする。
    fn check_acceptable(&self, metas: &[SpeakerMeta]) -> std::result::Result<(), StyleType> {
        let err = metas
            .iter()
            .flat_map(|SpeakerMeta { styles, .. }| styles)
            .map(|StyleMeta { r#type, .. }| *r#type)
            .unique()
            .find(|&style_type| !self.accepts(style_type));

        match err {
            Some(err) => Err(err),
            None => Ok(()),
        }
    }

    /// メタ情報にタイプが`style_type`のスタイルが含まれることを許容するかどうか。
    ///
    /// 例えば`self.talk`が`None`のとき、`StyleType::Talk`に対して`false`を返す。
    fn accepts(&self, style_type: StyleType) -> bool {
        let Self { talk } = self;

        return TalkDomain::contains(style_type).implies(|| talk.is_some());

        #[ext]
        impl<D: InferenceDomain> D {
            fn contains(style_type: StyleType) -> bool {
                Self::style_types().contains(&style_type)
            }
        }

        #[ext]
        impl bool {
            fn implies(self, other: impl FnOnce() -> Self) -> Self {
                !self || other()
            }
        }
    }
}

pub(crate) mod blocking {
    use std::path::Path;

    use easy_ext::ext;
    use uuid::Uuid;

    use crate::{
        asyncs::SingleTasked, error::LoadModelResult, future::FutureExt as _,
        infer::domains::InferenceDomainMap, VoiceModelMeta,
    };

    use super::{Inner, ModelBytesWithInnerVoiceIdsByDomain, VoiceModelHeader, VoiceModelId};

    /// 音声モデル。
    ///
    /// VVMファイルと対応する。
    pub struct VoiceModel(Inner<SingleTasked>);

    impl self::VoiceModel {
        pub(crate) fn read_inference_models(
            &self,
        ) -> LoadModelResult<InferenceDomainMap<ModelBytesWithInnerVoiceIdsByDomain>> {
            self.0.read_inference_models().block_on()
        }

        /// VVMファイルから`VoiceModel`をコンストラクトする。
        pub fn from_path(path: impl AsRef<Path>) -> crate::Result<Self> {
            Inner::from_path(path).block_on().map(Self)
        }

        /// ID。
        pub fn id(&self) -> VoiceModelId {
            self.0.id()
        }

        /// メタ情報。
        pub fn metas(&self) -> &VoiceModelMeta {
            self.0.metas()
        }

        pub(crate) fn header(&self) -> &VoiceModelHeader {
            self.0.header()
        }
    }

    #[ext(IdRef)]
    pub impl VoiceModel {
        fn id_ref(&self) -> &Uuid {
            &self.header().manifest.id.0
        }
    }
}

pub(crate) mod nonblocking {
    use std::path::Path;

    use crate::{
        asyncs::BlockingThreadPool, error::LoadModelResult, infer::domains::InferenceDomainMap,
        Result, VoiceModelMeta,
    };

    use super::{Inner, ModelBytesWithInnerVoiceIdsByDomain, VoiceModelHeader, VoiceModelId};

    /// 音声モデル。
    ///
    /// VVMファイルと対応する。
    ///
    /// # Performance
    ///
    /// [blocking]クレートにより動いている。詳しくは[`nonblocking`モジュールのドキュメント]を参照。
    ///
    /// [blocking]: https://docs.rs/crate/blocking
    /// [`nonblocking`モジュールのドキュメント]: crate::nonblocking
    pub struct VoiceModel(Inner<BlockingThreadPool>);

    impl self::VoiceModel {
        pub(crate) async fn read_inference_models(
            &self,
        ) -> LoadModelResult<InferenceDomainMap<ModelBytesWithInnerVoiceIdsByDomain>> {
            self.0.read_inference_models().await
        }
        /// VVMファイルから`VoiceModel`をコンストラクトする。
        pub async fn from_path(path: impl AsRef<Path>) -> Result<Self> {
            Inner::from_path(path).await.map(Self)
        }

        /// ID。
        pub fn id(&self) -> VoiceModelId {
            self.0.id()
        }

        /// メタ情報。
        pub fn metas(&self) -> &VoiceModelMeta {
            self.0.metas()
        }

        pub(crate) fn header(&self) -> &VoiceModelHeader {
            self.0.header()
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::{fixture, rstest};
    use serde_json::json;

    use crate::{
        infer::domains::InferenceDomainMap,
        manifest::{ManifestDomains, TalkManifest},
        SpeakerMeta, StyleType,
    };

    #[rstest]
    #[case(
        &InferenceDomainMap {
            talk: None,
        },
        &[],
        Ok(())
    )]
    #[case(
        &InferenceDomainMap {
            talk: Some(TalkManifest::default()),
        },
        &[speaker(&[StyleType::Talk])],
        Ok(())
    )]
    #[case(
        &InferenceDomainMap {
            talk: Some(TalkManifest::default()),
        },
        &[speaker(&[StyleType::Talk, StyleType::Sing])],
        Ok(())
    )]
    #[case(
        &InferenceDomainMap {
            talk: None,
        },
        &[speaker(&[StyleType::Talk])],
        Err(())
    )]
    fn check_acceptable_works(
        #[case] manifest: &InferenceDomainMap<ManifestDomains>,
        #[case] metas: &[SpeakerMeta],
        #[case] expected: std::result::Result<(), ()>,
    ) {
        let actual = manifest.check_acceptable(metas).map_err(|_| ());
        assert_eq!(expected, actual);
    }

    // FIXME: これ使ってないのでは？
    #[fixture]
    fn talk_speaker() -> SpeakerMeta {
        serde_json::from_value(json!({
            "name": "dummy",
            "styles": [
                {
                    "id": 0,
                    "name": "style1",
                    "type": "talk",
                    "order": 0
                }
            ],
            "version": "0.0.1",
            "speaker_uuid": "574bc678-8370-44be-b941-08e46e7b47d7",
            "order": 0
        }))
        .unwrap()
    }

    fn speaker(style_types: &'static [StyleType]) -> SpeakerMeta {
        let styles = style_types
            .iter()
            .map(|style_type| {
                json!({
                    "id": 0,
                    "name": "style1",
                    "type": style_type,
                    "order": null
                })
            })
            .collect::<Vec<_>>();

        serde_json::from_value(json!({
            "name": "dummy",
            "styles": styles,
            "version": "0.0.1",
            "speaker_uuid": "574bc678-8370-44be-b941-08e46e7b47d7",
            "order": null
        }))
        .unwrap()
    }
}
