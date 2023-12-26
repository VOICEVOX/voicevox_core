use derive_getters::Getters;
use derive_new::new;
use serde::Deserialize;

use crate::{
    manifest::{Manifest, ModelInnerId},
    SpeakerMeta, StyleId, StyleMeta, VoiceModelMeta,
};
use std::{collections::BTreeMap, path::PathBuf};

/// [`VoiceModelId`]の実体。
///
/// [`VoiceModelId`]: VoiceModelId
pub type RawVoiceModelId = String;

/// 音声モデルID。
#[derive(
    PartialEq, Eq, Clone, Ord, PartialOrd, Deserialize, new, Getters, derive_more::Display, Debug,
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

impl VoiceModelHeader {
    /// モデル内のすべてのスタイルに対するモデル内IDを取得する。
    ///
    /// モデル内IDのマッピングが存在しない場合はそのままスタイルIDを返す。
    pub(crate) fn model_inner_ids(&self) -> BTreeMap<StyleId, ModelInnerId> {
        self.metas
            .iter()
            .flat_map(SpeakerMeta::styles)
            .map(StyleMeta::id)
            .map(|&style_id| {
                let model_inner_id = self
                    .manifest
                    .style_id_to_model_inner_id()
                    .get(&style_id)
                    .copied()
                    .unwrap_or_else(|| ModelInnerId::new(style_id.raw_id()));
                (style_id, model_inner_id)
            })
            .collect()
    }
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
        infer::domain::InferenceOperationImpl,
        manifest::Manifest,
        VoiceModelMeta,
    };

    use super::{VoiceModelHeader, VoiceModelId};

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
        ) -> LoadModelResult<EnumMap<InferenceOperationImpl, Vec<u8>>> {
            let reader = BlockingVvmEntryReader::open(&self.header.path)?;

            let model_bytes = [
                self.header.manifest.predict_duration_filename(),
                self.header.manifest.predict_intonation_filename(),
                self.header.manifest.decode_filename(),
            ]
            .into_par_iter()
            .map(|filename| reader.read_vvm_entry(filename))
            .collect::<std::result::Result<Vec<_>, _>>()?
            .try_into()
            .unwrap_or_else(|_| panic!("should be same length"));

            Ok(EnumMap::from_array(model_bytes))
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
    use futures::future::join3;
    use nanoid::nanoid;
    use serde::de::DeserializeOwned;

    use crate::{
        error::{LoadModelError, LoadModelErrorKind, LoadModelResult},
        infer::domain::InferenceOperationImpl,
        manifest::Manifest,
        Result, VoiceModelMeta,
    };

    use super::{VoiceModelHeader, VoiceModelId};

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
        ) -> LoadModelResult<EnumMap<InferenceOperationImpl, Vec<u8>>> {
            let reader = AsyncVvmEntryReader::open(&self.header.path).await?;
            let (
                decode_model_result,
                predict_duration_model_result,
                predict_intonation_model_result,
            ) = join3(
                reader.read_vvm_entry(self.header.manifest.decode_filename()),
                reader.read_vvm_entry(self.header.manifest.predict_duration_filename()),
                reader.read_vvm_entry(self.header.manifest.predict_intonation_filename()),
            )
            .await;

            Ok(EnumMap::from_array([
                predict_duration_model_result?,
                predict_intonation_model_result?,
                decode_model_result?,
            ]))
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
    struct AsyncVvmEntryReader {
        reader: async_zip::read::fs::ZipFileReader,
        entry_map: HashMap<String, AsyncVvmEntry>,
    }

    impl AsyncVvmEntryReader {
        async fn open(path: &Path) -> LoadModelResult<Self> {
            let reader = async_zip::read::fs::ZipFileReader::new(path)
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
                .filter(|e| !e.entry().dir())
                .enumerate()
                .map(|(i, e)| {
                    (
                        e.entry().filename().to_string(),
                        AsyncVvmEntry {
                            index: i,
                            entry: e.entry().clone(),
                        },
                    )
                })
                .collect();
            Ok(AsyncVvmEntryReader::new(reader, entry_map))
        }
        async fn read_vvm_json<T: DeserializeOwned>(&self, filename: &str) -> LoadModelResult<T> {
            let bytes = self.read_vvm_entry(filename).await?;
            serde_json::from_slice(&bytes).map_err(|source| LoadModelError {
                path: self.reader.path().to_owned(),
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
                let mut manifest_reader = self.reader.entry(me.index).await?;
                let mut buf = Vec::with_capacity(me.entry.uncompressed_size() as usize);
                manifest_reader
                    .read_to_end_checked(&mut buf, &me.entry)
                    .await?;
                Ok::<_, anyhow::Error>(buf)
            }
            .await
            .map_err(|source| LoadModelError {
                path: self.reader.path().to_owned(),
                context: LoadModelErrorKind::ReadZipEntry {
                    filename: filename.to_owned(),
                },
                source: Some(source),
            })
        }
    }
}
