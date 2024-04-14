//! 音声モデル（ VVM ファイル）。
//!
//! VVM ファイルの定義と形式は[ドキュメント](../../../docs/vvm.md)を参照。

use derive_getters::Getters;
use derive_new::new;
use enum_map::EnumMap;
use serde::Deserialize;

use crate::{
    infer::domains::TalkOperation,
    manifest::{Manifest, StyleIdToModelInnerId},
    VoiceModelMeta,
};
use std::path::PathBuf;

/// [`VoiceModelId`]の実体。
///
/// [`VoiceModelId`]: VoiceModelId
pub type RawVoiceModelId = String;

pub(crate) type ModelBytesWithInnerIdsByDomain =
    (Option<(StyleIdToModelInnerId, EnumMap<TalkOperation, Vec<u8>>)>,);

/// 音声モデルID。
#[derive(
    PartialEq,
    Eq,
    Clone,
    Ord,
    Hash,
    PartialOrd,
    Deserialize,
    new,
    Getters,
    derive_more::Display,
    Debug,
)]
pub struct VoiceModelId {
    raw_voice_model_id: RawVoiceModelId,
}

// FIXME: "header"といいつつ、VVMのファイルパスを持っている状態になっている。
/// 音声モデルが持つ、各モデルファイルの実体を除く情報。
///
/// モデルの`[u8]`と分けて`Status`に渡す。
#[derive(Clone)]
pub(crate) struct VoiceModelHeader {
    /// ID。
    pub(crate) id: VoiceModelId,
    manifest: Manifest,
    /// メタ情報。
    pub(crate) metas: VoiceModelMeta,
    pub(crate) path: PathBuf,
}

pub(crate) mod blocking {
    use std::{
        io::{self, Cursor},
        path::Path,
    };

    use enum_map::EnumMap;
    use nanoid::nanoid;
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
                .talk()
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
            let path = path.as_ref().to_owned();
            let reader = BlockingVvmEntryReader::open(&path)?;
            let manifest = reader.read_vvm_json::<Manifest>("manifest.json")?;
            let metas = reader.read_vvm_json(manifest.metas_filename())?;
            let id = VoiceModelId::new(nanoid!());

            Ok(Self {
                header: VoiceModelHeader {
                    id,
                    metas,
                    manifest,
                    path,
                },
            })
        }

        /// ID。
        pub fn id(&self) -> &VoiceModelId {
            &self.header.id
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

        fn read_vvm_json<T: DeserializeOwned>(&self, filename: &str) -> LoadModelResult<T> {
            let bytes = &self.read_vvm_entry(filename)?;
            serde_json::from_slice(bytes).map_err(|source| LoadModelError {
                path: self.borrow_path().clone(),
                context: LoadModelErrorKind::OpenZipFile,
                source: Some(source.into()),
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
}

pub(crate) mod tokio {
    use std::{collections::HashMap, io, path::Path};

    use derive_new::new;
    use enum_map::EnumMap;
    use futures::future::{join3, OptionFuture};
    use nanoid::nanoid;
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

            let talk = OptionFuture::from(self.header.manifest.talk().as_ref().map(
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
            let metas = reader
                .read_vvm_json::<VoiceModelMeta>(manifest.metas_filename())
                .await?;
            let id = VoiceModelId::new(nanoid!());

            Ok(Self {
                header: VoiceModelHeader {
                    id,
                    metas,
                    manifest,
                    path: path.as_ref().into(),
                },
            })
        }

        /// ID。
        pub fn id(&self) -> &VoiceModelId {
            &self.header.id
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
        async fn read_vvm_json<T: DeserializeOwned>(&self, filename: &str) -> LoadModelResult<T> {
            let bytes = self.read_vvm_entry(filename).await?;
            serde_json::from_slice(&bytes).map_err(|source| LoadModelError {
                path: self.path.to_owned(),
                context: LoadModelErrorKind::ReadZipEntry {
                    filename: filename.to_owned(),
                },
                source: Some(source.into()),
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
