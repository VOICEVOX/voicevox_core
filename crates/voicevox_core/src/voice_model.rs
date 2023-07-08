use async_zip::{read::fs::ZipFileReader, ZipEntry};
use futures::future::{join3, join_all};
use serde::{de::DeserializeOwned, Deserialize};

use super::*;
use std::{
    collections::HashMap,
    env,
    path::{Path, PathBuf},
};

/// 音声モデルIdの実体
pub type RawVoiceModelId = String;

/// 音声モデルId (型を強く分けるためにこうしている)
#[derive(PartialEq, Eq, Clone, Ord, PartialOrd, Deserialize, new, Getters, Debug)]
pub struct VoiceModelId {
    raw_voice_model_id: RawVoiceModelId,
}

/// 音声モデル
#[derive(Getters, Clone)]
pub struct VoiceModel {
    id: VoiceModelId,
    manifest: Manifest,
    metas: VoiceModelMeta,
    path: PathBuf,
}

#[derive(Getters)]
pub(crate) struct InferenceModels {
    decode_model: Vec<u8>,
    predict_duration_model: Vec<u8>,
    predict_intonation_model: Vec<u8>,
}

impl VoiceModel {
    pub(crate) async fn read_inference_models(&self) -> Result<InferenceModels> {
        let reader = VvmEntryReader::open(&self.path).await?;
        let (decode_model_result, predict_duration_model_result, predict_intonation_model_result) =
            join3(
                reader.read_vvm_entry(self.manifest.decode_filename()),
                reader.read_vvm_entry(self.manifest.predict_duration_filename()),
                reader.read_vvm_entry(self.manifest.predict_intonation_filename()),
            )
            .await;

        Ok(InferenceModels {
            predict_duration_model: predict_duration_model_result?,
            predict_intonation_model: predict_intonation_model_result?,
            decode_model: decode_model_result?,
        })
    }
    /// 与えられたパスからモデルを取得する
    pub async fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let reader = VvmEntryReader::open(&path).await?;
        let manifest = reader.read_vvm_json::<Manifest>("manifest.json").await?;
        let metas = reader
            .read_vvm_json::<VoiceModelMeta>(manifest.metas_filename())
            .await?;
        let id = VoiceModelId::new(nanoid!());

        Ok(Self {
            id,
            metas,
            manifest,
            path: path.as_ref().into(),
        })
    }

    pub async fn get_all_models() -> Result<Vec<Self>> {
        let root_dir = if cfg!(test) {
            Path::new(env!("CARGO_WORKSPACE_DIR")).join("model")
        } else if let Some(root_dir) = env::var_os(Self::ROOT_DIR_ENV_NAME) {
            root_dir.into()
        } else {
            process_path::get_dylib_path()
                .or_else(process_path::get_executable_path)
                .unwrap()
                .parent()
                .unwrap_or_else(|| "".as_ref())
                .join("model")
        };

        let mut vvm_paths = Vec::new();
        for entry in root_dir.read_dir().map_err(|e| Error::LoadModel {
            path: root_dir.clone(),
            source: e.into(),
        })? {
            match entry {
                Ok(entry) => vvm_paths.push(Self::from_path(entry.path())),
                Err(e) => Err(Error::LoadModel {
                    path: root_dir.clone(),
                    source: e.into(),
                })?,
            }
        }

        join_all(vvm_paths).await.into_iter().collect()
    }
    const ROOT_DIR_ENV_NAME: &str = "VV_MODELS_ROOT_DIR";
}

struct VvmEntry {
    index: usize,
    entry: ZipEntry,
}

#[derive(new)]
struct VvmEntryReader {
    reader: ZipFileReader,
    entry_map: HashMap<String, VvmEntry>,
}

impl VvmEntryReader {
    async fn open(path: impl AsRef<Path>) -> Result<Self> {
        let reader = ZipFileReader::new(path.as_ref())
            .await
            .map_err(|e| Error::OpenFile {
                path: path.as_ref().into(),
                source: e.into(),
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
                    VvmEntry {
                        index: i,
                        entry: e.entry().clone(),
                    },
                )
            })
            .collect();
        Ok(VvmEntryReader::new(reader, entry_map))
    }
    async fn read_vvm_json<T: DeserializeOwned>(&self, filename: &str) -> Result<T> {
        let bytes = self.read_vvm_entry(filename).await?;
        serde_json::from_slice(&bytes).map_err(|e| Error::VvmRead {
            filename: filename.into(),
            source: Some(e.into()),
        })
    }

    async fn read_vvm_entry(&self, filename: &str) -> Result<Vec<u8>> {
        let me = self.entry_map.get(filename).ok_or(Error::VvmRead {
            filename: filename.into(),
            source: None,
        })?;
        let mut manifest_reader =
            self.reader
                .entry(me.index)
                .await
                .map_err(|_| Error::VvmRead {
                    filename: filename.into(),
                    source: None,
                })?;
        let mut buf = Vec::with_capacity(me.entry.uncompressed_size() as usize);
        manifest_reader
            .read_to_end_checked(&mut buf, &me.entry)
            .await
            .map_err(|_| Error::VvmRead {
                filename: filename.into(),
                source: None,
            })?;
        Ok(buf)
    }
}
