use async_zip::{read::fs::ZipFileReader, ZipEntry};
use enum_map::EnumMap;
use futures::future::join3;
use serde::{de::DeserializeOwned, Deserialize};

use super::*;
use crate::infer::domain::InferenceOperationKind;
use std::{
    collections::{BTreeMap, HashMap},
    io,
    path::{Path, PathBuf},
};

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

/// 音声モデル。
///
/// VVMファイルと対応する。
#[derive(Getters, Clone)]
pub struct VoiceModel {
    /// ID。
    id: VoiceModelId,
    manifest: Manifest,
    /// メタ情報。
    metas: VoiceModelMeta,
    path: PathBuf,
}

impl VoiceModel {
    pub(crate) async fn read_inference_models(
        &self,
    ) -> LoadModelResult<EnumMap<InferenceOperationKind, Vec<u8>>> {
        let reader = VvmEntryReader::open(&self.path).await?;
        let (decode_model_result, predict_duration_model_result, predict_intonation_model_result) =
            join3(
                reader.read_vvm_entry(self.manifest.decode_filename()),
                reader.read_vvm_entry(self.manifest.predict_duration_filename()),
                reader.read_vvm_entry(self.manifest.predict_intonation_filename()),
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
        let reader = VvmEntryReader::open(path.as_ref()).await?;
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
    async fn open(path: &Path) -> LoadModelResult<Self> {
        let reader = ZipFileReader::new(path)
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
                    VvmEntry {
                        index: i,
                        entry: e.entry().clone(),
                    },
                )
            })
            .collect();
        Ok(VvmEntryReader::new(reader, entry_map))
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
