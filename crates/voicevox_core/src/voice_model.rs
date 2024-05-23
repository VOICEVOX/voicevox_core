//! 音声モデル（ VVM ファイル）。
//!
//! VVM ファイルの定義と形式は[ドキュメント](../../../docs/vvm.md)を参照。

use anyhow::anyhow;
use derive_getters::Getters;
use derive_more::From;
use derive_new::new;
use easy_ext::ext;
use enum_map::EnumMap;
use itertools::Itertools as _;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    error::{LoadModelError, LoadModelErrorKind, LoadModelResult},
    infer::{
        domains::{TalkDomain, TalkOperation},
        InferenceDomain,
    },
    manifest::{Manifest, ManifestDomains, StyleIdToModelInnerId},
    SpeakerMeta, StyleMeta, StyleType, VoiceModelMeta,
};
use std::path::{Path, PathBuf};

/// [`VoiceModelId`]の実体。
///
/// [`VoiceModelId`]: VoiceModelId
pub type RawVoiceModelId = Uuid;

pub(crate) type ModelBytesWithInnerIdsByDomain =
    (Option<(StyleIdToModelInnerId, EnumMap<TalkOperation, Vec<u8>>)>,);

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
    new,
    Getters,
    derive_more::Display,
    Debug,
    From,
)]
#[serde(transparent)]
pub struct VoiceModelId {
    raw_voice_model_id: RawVoiceModelId,
}

// FIXME: "header"といいつつ、VVMのファイルパスを持っている状態になっている。
/// 音声モデルが持つ、各モデルファイルの実体を除く情報。
///
/// モデルの`[u8]`と分けて`Status`に渡す。
#[derive(Clone)]
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
        let metas =
            serde_json::from_slice::<VoiceModelMeta>(metas).map_err(|source| LoadModelError {
                path: path.to_owned(),
                context: LoadModelErrorKind::InvalidModelFormat,
                source: Some(
                    anyhow::Error::from(source)
                        .context(format!("{}が不正です", manifest.metas_filename())),
                ),
            })?;

        manifest
            .domains()
            .check_acceptable(&metas)
            .map_err(|style_type| LoadModelError {
                path: path.to_owned(),
                context: LoadModelErrorKind::InvalidModelFormat,
                source: Some(anyhow!(
                    "{metas_filename}には`{style_type}`のスタイルが存在しますが、manifest.jsonでの\
                     対応がありません",
                    metas_filename = manifest.metas_filename(),
                )),
            })?;

        Ok(Self {
            manifest,
            metas,
            path: path.to_owned(),
        })
    }
}

impl ManifestDomains {
    /// manifestとして対応していない`StyleType`に対してエラーを発する。
    ///
    /// `Status`はこのバリデーションを信頼し、`InferenceDomain`の不足時にパニックする。
    fn check_acceptable(&self, metas: &[SpeakerMeta]) -> std::result::Result<(), StyleType> {
        let err = metas
            .iter()
            .flat_map(SpeakerMeta::styles)
            .map(StyleMeta::r#type)
            .copied()
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
    use std::{
        io::{self, Cursor},
        path::Path,
    };

    use easy_ext::ext;
    use enum_map::EnumMap;
    use ouroboros::self_referencing;
    use rayon::iter::{IntoParallelIterator as _, ParallelIterator as _};
    use serde::de::DeserializeOwned;

    use crate::{
        error::{LoadModelError, LoadModelErrorKind, LoadModelResult},
        infer::domains::InferenceDomainMap,
        manifest::{Manifest, TalkManifest},
        VoiceModelMeta,
    };

    use super::{ModelBytesWithInnerIdsByDomain, VoiceModelHeader, VoiceModelId};

    /// 音声モデル。
    ///
    /// VVMファイルと対応する。
    #[derive(Clone)]
    pub struct VoiceModel {
        header: VoiceModelHeader,
    }

    impl self::VoiceModel {
        pub(crate) fn read_inference_models(
            &self,
        ) -> LoadModelResult<InferenceDomainMap<ModelBytesWithInnerIdsByDomain>> {
            let reader = BlockingVvmEntryReader::open(&self.header.path)?;

            let talk = self
                .header
                .manifest
                .domains()
                .talk
                .as_ref()
                .map(
                    |TalkManifest {
                         predict_duration_filename,
                         predict_intonation_filename,
                         decode_filename,
                         style_id_to_model_inner_id,
                     }| {
                        let model_bytes = [
                            predict_duration_filename,
                            predict_intonation_filename,
                            decode_filename,
                        ]
                        .into_par_iter()
                        .map(|filename| reader.read_vvm_entry(filename))
                        .collect::<std::result::Result<Vec<_>, _>>()?
                        .try_into()
                        .unwrap_or_else(|_| panic!("should be same length"));

                        let model_bytes = EnumMap::from_array(model_bytes);

                        Ok((style_id_to_model_inner_id.clone(), model_bytes))
                    },
                )
                .transpose()?;

            Ok(InferenceDomainMap { talk })
        }

        /// VVMファイルから`VoiceModel`をコンストラクトする。
        pub fn from_path(path: impl AsRef<Path>) -> crate::Result<Self> {
            let path = path.as_ref();
            let reader = BlockingVvmEntryReader::open(path)?;
            let manifest = reader.read_vvm_json::<Manifest>("manifest.json")?;
            let metas = &reader.read_vvm_entry(manifest.metas_filename())?;
            let header = VoiceModelHeader::new(manifest, metas, path)?;
            Ok(Self { header })
        }

        /// ID。
        pub fn id(&self) -> VoiceModelId {
            self.header.manifest.id
        }

        /// メタ情報。
        pub fn metas(&self) -> &VoiceModelMeta {
            &self.header.metas
        }

        pub(crate) fn header(&self) -> &VoiceModelHeader {
            &self.header
        }
    }

    #[self_referencing]
    struct BlockingVvmEntryReader {
        path: std::path::PathBuf,
        zip: Vec<u8>,
        #[covariant]
        #[borrows(zip)]
        reader: zip::ZipArchive<Cursor<&'this [u8]>>,
    }

    impl BlockingVvmEntryReader {
        fn open(path: &Path) -> LoadModelResult<Self> {
            (|| {
                let zip = std::fs::read(path)?;
                Self::try_new(path.to_owned(), zip, |zip| {
                    zip::ZipArchive::new(Cursor::new(zip))
                })
            })()
            .map_err(|source| LoadModelError {
                path: path.to_owned(),
                context: LoadModelErrorKind::OpenZipFile,
                source: Some(source.into()),
            })
        }

        // FIXME: manifest.json専用になっているので、そういう関数名にする
        fn read_vvm_json<T: DeserializeOwned>(&self, filename: &str) -> LoadModelResult<T> {
            let bytes = &self.read_vvm_entry(filename)?;
            serde_json::from_slice(bytes).map_err(|source| LoadModelError {
                path: self.borrow_path().clone(),
                context: LoadModelErrorKind::InvalidModelFormat,
                source: Some(anyhow::Error::from(source).context(format!("{filename}が不正です"))),
            })
        }

        fn read_vvm_entry(&self, filename: &str) -> LoadModelResult<Vec<u8>> {
            (|| {
                let mut reader = self.borrow_reader().clone();
                let mut entry = reader.by_name(filename)?;
                let mut buf = Vec::with_capacity(entry.size() as _);
                io::copy(&mut entry, &mut buf)?;
                Ok(buf)
            })()
            .map_err(|source| LoadModelError {
                path: self.borrow_path().clone(),
                context: LoadModelErrorKind::OpenZipFile,
                source: Some(source),
            })
        }
    }

    #[ext(IdRef)]
    pub impl VoiceModel {
        fn id_ref(&self) -> &VoiceModelId {
            &self.header.manifest.id
        }
    }
}

pub(crate) mod tokio {
    use std::{collections::HashMap, io, path::Path};

    use derive_new::new;
    use enum_map::EnumMap;
    use futures::future::{join3, OptionFuture};
    use serde::de::DeserializeOwned;

    use crate::{
        error::{LoadModelError, LoadModelErrorKind, LoadModelResult},
        infer::domains::InferenceDomainMap,
        manifest::{Manifest, TalkManifest},
        Result, VoiceModelMeta,
    };

    use super::{ModelBytesWithInnerIdsByDomain, VoiceModelHeader, VoiceModelId};

    /// 音声モデル。
    ///
    /// VVMファイルと対応する。
    #[derive(Clone)]
    pub struct VoiceModel {
        header: VoiceModelHeader,
    }

    impl self::VoiceModel {
        pub(crate) async fn read_inference_models(
            &self,
        ) -> LoadModelResult<InferenceDomainMap<ModelBytesWithInnerIdsByDomain>> {
            let reader = AsyncVvmEntryReader::open(&self.header.path).await?;

            let talk = OptionFuture::from(self.header.manifest.domains().talk.as_ref().map(
                |TalkManifest {
                     predict_duration_filename,
                     predict_intonation_filename,
                     decode_filename,
                     style_id_to_model_inner_id,
                 }| async {
                    let (
                        decode_model_result,
                        predict_duration_model_result,
                        predict_intonation_model_result,
                    ) = join3(
                        reader.read_vvm_entry(decode_filename),
                        reader.read_vvm_entry(predict_duration_filename),
                        reader.read_vvm_entry(predict_intonation_filename),
                    )
                    .await;

                    let model_bytes = EnumMap::from_array([
                        predict_duration_model_result?,
                        predict_intonation_model_result?,
                        decode_model_result?,
                    ]);

                    Ok((style_id_to_model_inner_id.clone(), model_bytes))
                },
            ))
            .await
            .transpose()?;

            Ok(InferenceDomainMap { talk })
        }
        /// VVMファイルから`VoiceModel`をコンストラクトする。
        pub async fn from_path(path: impl AsRef<Path>) -> Result<Self> {
            let reader = AsyncVvmEntryReader::open(path.as_ref()).await?;
            let manifest = reader.read_vvm_json::<Manifest>("manifest.json").await?;
            let metas = &reader.read_vvm_entry(manifest.metas_filename()).await?;
            let header = VoiceModelHeader::new(manifest, metas, path.as_ref())?;
            Ok(Self { header })
        }

        /// ID。
        pub fn id(&self) -> VoiceModelId {
            self.header.manifest.id
        }

        /// メタ情報。
        pub fn metas(&self) -> &VoiceModelMeta {
            &self.header.metas
        }

        pub(crate) fn header(&self) -> &VoiceModelHeader {
            &self.header
        }
    }

    struct AsyncVvmEntry {
        index: usize,
        entry: async_zip::ZipEntry,
    }

    #[derive(new)]
    struct AsyncVvmEntryReader<'a> {
        path: &'a Path,
        reader: async_zip::base::read::mem::ZipFileReader,
        entry_map: HashMap<String, AsyncVvmEntry>,
    }

    impl<'a> AsyncVvmEntryReader<'a> {
        async fn open(path: &'a Path) -> LoadModelResult<Self> {
            let reader = async {
                let file = fs_err::tokio::read(path).await?;
                async_zip::base::read::mem::ZipFileReader::new(file).await
            }
            .await
            .map_err(|source| LoadModelError {
                path: path.to_owned(),
                context: LoadModelErrorKind::OpenZipFile,
                source: Some(source.into()),
            })?;
            let entry_map: HashMap<_, _> = reader
                .file()
                .entries()
                .iter()
                .flat_map(|e| {
                    // 非UTF-8のファイルを利用することはないため、無視する
                    let filename = e.filename().as_str().ok()?;
                    (!e.dir().ok()?).then_some(())?;
                    Some((filename.to_owned(), (**e).clone()))
                })
                .enumerate()
                .map(|(i, (filename, entry))| (filename, AsyncVvmEntry { index: i, entry }))
                .collect();
            Ok(AsyncVvmEntryReader::new(path, reader, entry_map))
        }
        // FIXME: manifest.json専用になっているので、そういう関数名にする
        async fn read_vvm_json<T: DeserializeOwned>(&self, filename: &str) -> LoadModelResult<T> {
            let bytes = self.read_vvm_entry(filename).await?;
            serde_json::from_slice(&bytes).map_err(|source| LoadModelError {
                path: self.path.to_owned(),
                context: LoadModelErrorKind::InvalidModelFormat,
                source: Some(anyhow::Error::from(source).context(format!("{filename}が不正です"))),
            })
        }

        async fn read_vvm_entry(&self, filename: &str) -> LoadModelResult<Vec<u8>> {
            async {
                let me = self
                    .entry_map
                    .get(filename)
                    .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
                let mut manifest_reader = self.reader.reader_with_entry(me.index).await?;
                let mut buf = Vec::with_capacity(me.entry.uncompressed_size() as usize);
                manifest_reader.read_to_end_checked(&mut buf).await?;
                Ok::<_, anyhow::Error>(buf)
            }
            .await
            .map_err(|source| LoadModelError {
                path: self.path.to_owned(),
                context: LoadModelErrorKind::ReadZipEntry {
                    filename: filename.to_owned(),
                },
                source: Some(source),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use once_cell::sync::Lazy;
    use rstest::{fixture, rstest};
    use serde_json::json;

    use crate::{
        manifest::{ManifestDomains, TalkManifest},
        SpeakerMeta, StyleType,
    };

    #[rstest]
    #[case(
        &ManifestDomains {
            talk: None,
        },
        &[],
        Ok(())
    )]
    #[case(
        &ManifestDomains {
            talk: Some(TALK_MANIFEST.clone()),
        },
        &[speaker(&[StyleType::Talk])],
        Ok(())
    )]
    #[case(
        &ManifestDomains {
            talk: Some(TALK_MANIFEST.clone()),
        },
        &[speaker(&[StyleType::Talk, StyleType::Sing])],
        Ok(())
    )]
    #[case(
        &ManifestDomains {
            talk: None,
        },
        &[speaker(&[StyleType::Talk])],
        Err(())
    )]
    fn check_acceptable_works(
        #[case] manifest: &ManifestDomains,
        #[case] metas: &[SpeakerMeta],
        #[case] expected: std::result::Result<(), ()>,
    ) {
        let actual = manifest.check_acceptable(metas).map_err(|_| ());
        assert_eq!(expected, actual);
    }

    static TALK_MANIFEST: Lazy<TalkManifest> = Lazy::new(|| TalkManifest {
        predict_duration_filename: "".to_owned(),
        predict_intonation_filename: "".to_owned(),
        decode_filename: "".to_owned(),
        style_id_to_model_inner_id: Default::default(),
    });

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
