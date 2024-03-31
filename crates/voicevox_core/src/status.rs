use super::*;
use anyhow::Context as _;
use once_cell::sync::Lazy;
use onnxruntime::{
    environment::Environment,
    session::{AnyArray, Session},
    GraphOptimizationLevel, LoggingLevel,
};
use serde::{Deserialize, Serialize};
use std::{
    env,
    path::{Path, PathBuf},
};
use tracing::error;

mod model_file;

cfg_if! {
    if #[cfg(not(feature="directml"))]{
        use onnxruntime::CudaProviderOptions;
    }
}
use std::collections::{BTreeMap, BTreeSet};

pub(crate) static MODEL_FILE_SET: Lazy<ModelFileSet> = Lazy::new(|| {
    let result = ModelFileSet::new();
    if let Err(err) = &result {
        error!("ファイルを読み込めなかったためクラッシュします: {err}");
    }
    result.unwrap()
});

pub struct Status {
    talk_models: StatusTalkModels,
    sing_teacher_models: StatusSingTeacherModels,
    sf_decode_models: StatusSfModels,
    light_session_options: SessionOptions, // 軽いモデルはこちらを使う
    heavy_session_options: SessionOptions, // 重いモデルはこちらを使う
    supported_styles: BTreeSet<u32>,
}

struct StatusTalkModels {
    predict_duration: BTreeMap<usize, Session<'static>>,
    predict_intonation: BTreeMap<usize, Session<'static>>,
    decode: BTreeMap<usize, Session<'static>>,
}

struct StatusSingTeacherModels {
    predict_sing_consonant_length: BTreeMap<usize, Session<'static>>,
    predict_sing_f0: BTreeMap<usize, Session<'static>>,
    predict_sing_volume: BTreeMap<usize, Session<'static>>,
}

struct StatusSfModels {
    sf_decode: BTreeMap<usize, Session<'static>>,
}

#[derive(new, Getters)]
struct SessionOptions {
    cpu_num_threads: u16,
    use_gpu: bool,
}

pub(crate) struct ModelFileSet {
    pub(crate) talk_speaker_id_map: BTreeMap<u32, (usize, u32)>,
    pub(crate) sing_teacher_speaker_id_map: BTreeMap<u32, (usize, u32)>,
    pub(crate) sf_decode_speaker_id_map: BTreeMap<u32, (usize, u32)>,
    pub(crate) metas_str: String,
    talk_models: Vec<TalkModel>,
    sing_teacher_models: Vec<SingTeacherModel>,
    sf_decode_models: Vec<SfDecodeModel>,
}

impl ModelFileSet {
    fn new() -> anyhow::Result<Self> {
        let path = {
            let root_dir = if cfg!(test) {
                Path::new(env!("CARGO_WORKSPACE_DIR")).join("model")
            } else if let Some(root_dir) = env::var_os(ROOT_DIR_ENV_NAME) {
                root_dir.into()
            } else {
                process_path::get_dylib_path()
                    .or_else(process_path::get_executable_path)
                    .with_context(|| "Could not get the current dynamic library/executable path")?
                    .parent()
                    .unwrap_or_else(|| "".as_ref())
                    .join("model")
            };

            move |rel_path| root_dir.join(rel_path)
        };

        let metas_str = fs_err::read_to_string(path("metas.json"))?;

        let talk_models = model_file::TALK_MODEL_FILE_NAMES
            .iter()
            .map(
                |&TalkModelFileNames {
                     predict_duration_model,
                     predict_intonation_model,
                     decode_model,
                 }| {
                    let predict_duration_model = path(predict_duration_model);
                    let predict_intonation_model = path(predict_intonation_model);
                    let decode_model = path(decode_model);
                    Ok(TalkModel {
                        predict_duration_model,
                        predict_intonation_model,
                        decode_model,
                    })
                },
            )
            .collect::<anyhow::Result<_>>()?;

        let sing_teacher_models = model_file::SING_TEACHER_MODEL_FILE_NAMES
            .iter()
            .map(
                |&SingTeacherModelFileNames {
                     predict_sing_consonant_length_model,
                     predict_sing_f0_model,
                     predict_sing_volume_model,
                 }| {
                    let predict_sing_consonant_length_model =
                        path(predict_sing_consonant_length_model);
                    let predict_sing_f0_model = path(predict_sing_f0_model);
                    let predict_sing_volume_model = path(predict_sing_volume_model);
                    Ok(SingTeacherModel {
                        predict_sing_consonant_length_model,
                        predict_sing_f0_model,
                        predict_sing_volume_model,
                    })
                },
            )
            .collect::<anyhow::Result<_>>()?;

        let sf_decode_models = model_file::SF_DECODE_MODEL_FILE_NAMES
            .iter()
            .map(|&SfDecodeModelFileNames { sf_decode_model }| {
                let sf_decode_model = path(sf_decode_model);
                Ok(SfDecodeModel { sf_decode_model })
            })
            .collect::<anyhow::Result<_>>()?;

        return Ok(Self {
            talk_speaker_id_map: model_file::TALK_SPEAKER_ID_MAP.iter().copied().collect(),
            sing_teacher_speaker_id_map: model_file::SING_TEACHER_SPEAKER_ID_MAP
                .iter()
                .copied()
                .collect(),
            sf_decode_speaker_id_map: model_file::SF_DECODE_SPEAKER_ID_MAP
                .iter()
                .copied()
                .collect(),
            metas_str,
            talk_models,
            sing_teacher_models,
            sf_decode_models,
        });

        const ROOT_DIR_ENV_NAME: &str = "VV_MODELS_ROOT_DIR";
    }

    pub(crate) fn talk_models_count(&self) -> usize {
        self.talk_models.len()
    }

    pub(crate) fn sing_teacher_models_count(&self) -> usize {
        self.sing_teacher_models.len()
    }

    pub(crate) fn sf_decode_models_count(&self) -> usize {
        self.sf_decode_models.len()
    }
}

struct TalkModelFileNames {
    predict_duration_model: &'static str,
    predict_intonation_model: &'static str,
    decode_model: &'static str,
}

struct SingTeacherModelFileNames {
    predict_sing_consonant_length_model: &'static str,
    predict_sing_f0_model: &'static str,
    predict_sing_volume_model: &'static str,
}

struct SfDecodeModelFileNames {
    sf_decode_model: &'static str,
}

#[derive(thiserror::Error, Debug)]
#[error("不正なモデルファイルです")]
struct DecryptModelError;

struct TalkModel {
    predict_duration_model: PathBuf,
    predict_intonation_model: PathBuf,
    decode_model: PathBuf,
}

struct SingTeacherModel {
    predict_sing_consonant_length_model: PathBuf,
    predict_sing_f0_model: PathBuf,
    predict_sing_volume_model: PathBuf,
}

struct SfDecodeModel {
    sf_decode_model: PathBuf,
}

#[derive(Deserialize, Getters)]
struct Meta {
    styles: Vec<Style>,
}

#[derive(Deserialize, Getters)]
struct Style {
    id: u64,
}
static ENVIRONMENT: Lazy<Environment> = Lazy::new(|| {
    cfg_if! {
        if #[cfg(debug_assertions)]{
            const LOGGING_LEVEL: LoggingLevel = LoggingLevel::Verbose;
        } else{
            const LOGGING_LEVEL: LoggingLevel = LoggingLevel::Warning;
        }
    }
    Environment::builder()
        .with_name(env!("CARGO_PKG_NAME"))
        .with_log_level(LOGGING_LEVEL)
        .build()
        .unwrap()
});

#[derive(Getters, Debug, Serialize, Deserialize)]
pub struct SupportedDevices {
    cpu: bool,
    cuda: bool,
    dml: bool,
}

impl SupportedDevices {
    pub fn get_supported_devices() -> Result<Self> {
        let mut cuda_support = false;
        let mut dml_support = false;
        for provider in onnxruntime::session::get_available_providers()
            .map_err(|e| Error::GetSupportedDevices(e.into()))?
            .iter()
        {
            match provider.as_str() {
                "CUDAExecutionProvider" => cuda_support = true,
                "DmlExecutionProvider" => dml_support = true,
                _ => {}
            }
        }

        Ok(SupportedDevices {
            cpu: true,
            cuda: cuda_support,
            dml: dml_support,
        })
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).expect("should not fail")
    }
}

#[allow(unsafe_code)]
unsafe impl Send for Status {}

impl Status {
    pub fn new(use_gpu: bool, cpu_num_threads: u16) -> Self {
        Self {
            talk_models: StatusTalkModels {
                predict_duration: BTreeMap::new(),
                predict_intonation: BTreeMap::new(),
                decode: BTreeMap::new(),
            },
            sing_teacher_models: StatusSingTeacherModels {
                predict_sing_consonant_length: BTreeMap::new(),
                predict_sing_f0: BTreeMap::new(),
                predict_sing_volume: BTreeMap::new(),
            },
            sf_decode_models: StatusSfModels {
                sf_decode: BTreeMap::new(),
            },
            light_session_options: SessionOptions::new(cpu_num_threads, false),
            heavy_session_options: SessionOptions::new(cpu_num_threads, use_gpu),
            supported_styles: BTreeSet::default(),
        }
    }

    pub fn load_metas(&mut self) -> Result<()> {
        let metas: Vec<Meta> = serde_json::from_str(&MODEL_FILE_SET.metas_str)
            .map_err(|e| Error::LoadMetas(e.into()))?;

        for meta in metas.iter() {
            for style in meta.styles().iter() {
                self.supported_styles.insert(*style.id() as u32);
            }
        }

        Ok(())
    }

    pub fn load_talk_model(&mut self, model_index: usize) -> Result<()> {
        if model_index < MODEL_FILE_SET.talk_models.len() {
            let model = &MODEL_FILE_SET.talk_models[model_index];
            let predict_duration_session =
                self.new_session(&model.predict_duration_model, &self.light_session_options)?;
            let predict_intonation_session =
                self.new_session(&model.predict_intonation_model, &self.light_session_options)?;
            let decode_model =
                self.new_session(&model.decode_model, &self.heavy_session_options)?;

            self.talk_models
                .predict_duration
                .insert(model_index, predict_duration_session);
            self.talk_models
                .predict_intonation
                .insert(model_index, predict_intonation_session);

            self.talk_models.decode.insert(model_index, decode_model);

            Ok(())
        } else {
            Err(Error::InvalidModelIndex { model_index })
        }
    }

    pub fn is_talk_model_loaded(&self, model_index: usize) -> bool {
        self.talk_models.predict_duration.contains_key(&model_index)
            && self
                .talk_models
                .predict_intonation
                .contains_key(&model_index)
            && self.talk_models.decode.contains_key(&model_index)
    }

    pub fn load_sing_teacher_model(&mut self, model_index: usize) -> Result<()> {
        if model_index < MODEL_FILE_SET.sing_teacher_models.len() {
            let model = &MODEL_FILE_SET.sing_teacher_models[model_index];
            let predict_sing_consonant_length_session = self.new_session(
                &model.predict_sing_consonant_length_model,
                &self.light_session_options,
            )?;
            let predict_sing_f0_session =
                self.new_session(&model.predict_sing_f0_model, &self.light_session_options)?;
            let predict_sing_volume_session = self.new_session(
                &model.predict_sing_volume_model,
                &self.light_session_options,
            )?;

            self.sing_teacher_models
                .predict_sing_consonant_length
                .insert(model_index, predict_sing_consonant_length_session);
            self.sing_teacher_models
                .predict_sing_f0
                .insert(model_index, predict_sing_f0_session);
            self.sing_teacher_models
                .predict_sing_volume
                .insert(model_index, predict_sing_volume_session);

            Ok(())
        } else {
            Err(Error::InvalidModelIndex { model_index })
        }
    }

    pub fn is_sing_teacher_model_loaded(&self, model_index: usize) -> bool {
        self.sing_teacher_models
            .predict_sing_consonant_length
            .contains_key(&model_index)
            && self
                .sing_teacher_models
                .predict_sing_f0
                .contains_key(&model_index)
            && self
                .sing_teacher_models
                .predict_sing_volume
                .contains_key(&model_index)
    }

    pub fn load_sf_decode_model(&mut self, model_index: usize) -> Result<()> {
        if model_index < MODEL_FILE_SET.sf_decode_models.len() {
            let model = &MODEL_FILE_SET.sf_decode_models[model_index];
            let sf_decode_session =
                self.new_session(&model.sf_decode_model, &self.heavy_session_options)?;

            self.sf_decode_models
                .sf_decode
                .insert(model_index, sf_decode_session);

            Ok(())
        } else {
            Err(Error::InvalidModelIndex { model_index })
        }
    }

    pub fn is_sf_decode_model_loaded(&self, model_index: usize) -> bool {
        self.sf_decode_models.sf_decode.contains_key(&model_index)
    }

    fn new_session(
        &self,
        model_file: &Path,
        session_options: &SessionOptions,
    ) -> Result<Session<'static>> {
        let model_bytes = &match fs_err::read(model_file) {
            Ok(model_bytes) => model_bytes,
            Err(err) => {
                panic!("ファイルを読み込めなかったためクラッシュします: {err}");
            }
        };
        self.new_session_from_bytes(|| model_file::decrypt(model_bytes), session_options)
            .map_err(|source| Error::LoadModel {
                path: model_file.to_owned(),
                source,
            })
    }

    fn new_session_from_bytes(
        &self,
        model_bytes: impl FnOnce() -> std::result::Result<Vec<u8>, DecryptModelError>,
        session_options: &SessionOptions,
    ) -> anyhow::Result<Session<'static>> {
        let session_builder = ENVIRONMENT
            .new_session_builder()?
            .with_optimization_level(GraphOptimizationLevel::Basic)?
            .with_intra_op_num_threads(*session_options.cpu_num_threads() as i32)?
            .with_inter_op_num_threads(*session_options.cpu_num_threads() as i32)?;

        let session_builder = if *session_options.use_gpu() {
            cfg_if! {
                if #[cfg(feature = "directml")]{
                    session_builder
                        .with_disable_mem_pattern()?
                        .with_execution_mode(onnxruntime::ExecutionMode::ORT_SEQUENTIAL)?
                        .with_append_execution_provider_directml(0)?
                } else {
                    let options = CudaProviderOptions::default();
                    session_builder.with_append_execution_provider_cuda(options)?
                }
            }
        } else {
            session_builder
        };

        Ok(session_builder.with_model_from_memory(model_bytes()?)?)
    }

    pub fn validate_speaker_id(&self, speaker_id: u32) -> bool {
        self.supported_styles.contains(&speaker_id)
    }

    pub fn predict_duration_session_run(
        &mut self,
        model_index: usize,
        inputs: Vec<&mut dyn AnyArray>,
    ) -> Result<Vec<f32>> {
        if let Some(model) = self.talk_models.predict_duration.get_mut(&model_index) {
            if let Ok(output_tensors) = model.run(inputs) {
                Ok(output_tensors[0].as_slice().unwrap().to_owned())
            } else {
                Err(Error::InferenceFailed)
            }
        } else {
            Err(Error::InvalidModelIndex { model_index })
        }
    }

    pub fn predict_intonation_session_run(
        &mut self,
        model_index: usize,
        inputs: Vec<&mut dyn AnyArray>,
    ) -> Result<Vec<f32>> {
        if let Some(model) = self.talk_models.predict_intonation.get_mut(&model_index) {
            if let Ok(output_tensors) = model.run(inputs) {
                Ok(output_tensors[0].as_slice().unwrap().to_owned())
            } else {
                Err(Error::InferenceFailed)
            }
        } else {
            Err(Error::InvalidModelIndex { model_index })
        }
    }

    pub fn decode_session_run(
        &mut self,
        model_index: usize,
        inputs: Vec<&mut dyn AnyArray>,
    ) -> Result<Vec<f32>> {
        if let Some(model) = self.talk_models.decode.get_mut(&model_index) {
            if let Ok(output_tensors) = model.run(inputs) {
                Ok(output_tensors[0].as_slice().unwrap().to_owned())
            } else {
                Err(Error::InferenceFailed)
            }
        } else {
            Err(Error::InvalidModelIndex { model_index })
        }
    }

    pub fn predict_sing_consonant_length_session_run(
        &mut self,
        model_index: usize,
        inputs: Vec<&mut dyn AnyArray>,
    ) -> Result<Vec<i64>> {
        if let Some(model) = self
            .sing_teacher_models
            .predict_sing_consonant_length
            .get_mut(&model_index)
        {
            if let Ok(output_tensors) = model.run(inputs) {
                Ok(output_tensors[0].as_slice().unwrap().to_owned())
            } else {
                Err(Error::InferenceFailed)
            }
        } else {
            Err(Error::InvalidModelIndex { model_index })
        }
    }

    pub fn predict_sing_f0_session_run(
        &mut self,
        model_index: usize,
        inputs: Vec<&mut dyn AnyArray>,
    ) -> Result<Vec<f32>> {
        if let Some(model) = self
            .sing_teacher_models
            .predict_sing_f0
            .get_mut(&model_index)
        {
            if let Ok(output_tensors) = model.run(inputs) {
                Ok(output_tensors[0].as_slice().unwrap().to_owned())
            } else {
                Err(Error::InferenceFailed)
            }
        } else {
            Err(Error::InvalidModelIndex { model_index })
        }
    }

    pub fn predict_sing_volume_session_run(
        &mut self,
        model_index: usize,
        inputs: Vec<&mut dyn AnyArray>,
    ) -> Result<Vec<f32>> {
        if let Some(model) = self
            .sing_teacher_models
            .predict_sing_volume
            .get_mut(&model_index)
        {
            if let Ok(output_tensors) = model.run(inputs) {
                Ok(output_tensors[0].as_slice().unwrap().to_owned())
            } else {
                Err(Error::InferenceFailed)
            }
        } else {
            Err(Error::InvalidModelIndex { model_index })
        }
    }

    pub fn sf_decode_session_run(
        &mut self,
        model_index: usize,
        inputs: Vec<&mut dyn AnyArray>,
    ) -> Result<Vec<f32>> {
        if let Some(model) = self.sf_decode_models.sf_decode.get_mut(&model_index) {
            if let Ok(output_tensors) = model.run(inputs) {
                Ok(output_tensors[0].as_slice().unwrap().to_owned())
            } else {
                Err(Error::InferenceFailed)
            }
        } else {
            Err(Error::InvalidModelIndex { model_index })
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use pretty_assertions::assert_eq;

    #[rstest]
    #[case(true, 0)]
    #[case(true, 1)]
    #[case(true, 8)]
    #[case(false, 2)]
    #[case(false, 4)]
    #[case(false, 8)]
    #[case(false, 0)]
    fn status_new_works(#[case] use_gpu: bool, #[case] cpu_num_threads: u16) {
        let status = Status::new(use_gpu, cpu_num_threads);
        assert_eq!(false, status.light_session_options.use_gpu);
        assert_eq!(use_gpu, status.heavy_session_options.use_gpu);
        assert_eq!(
            cpu_num_threads,
            status.light_session_options.cpu_num_threads
        );
        assert_eq!(
            cpu_num_threads,
            status.heavy_session_options.cpu_num_threads
        );
        assert!(status.talk_models.predict_duration.is_empty());
        assert!(status.talk_models.predict_intonation.is_empty());
        assert!(status.talk_models.decode.is_empty());
        assert!(status
            .sing_teacher_models
            .predict_sing_consonant_length
            .is_empty());
        assert!(status.sing_teacher_models.predict_sing_f0.is_empty());
        assert!(status.sing_teacher_models.predict_sing_volume.is_empty());
        assert!(status.sf_decode_models.sf_decode.is_empty());
        assert!(status.supported_styles.is_empty());
    }

    #[rstest]
    fn status_load_metas_works() {
        let mut status = Status::new(true, 0);
        let result = status.load_metas();
        assert_eq!(Ok(()), result);
        let expected = BTreeSet::from([0, 1, 2, 3, 3000, 6000]);
        assert_eq!(expected, status.supported_styles);
    }

    #[rstest]
    fn supported_devices_get_supported_devices_works() {
        let result = SupportedDevices::get_supported_devices();
        // 環境によって結果が変わるので、関数呼び出しが成功するかどうかの確認のみ行う
        assert!(result.is_ok(), "{result:?}");
    }

    #[rstest]
    fn status_load_talk_model_works() {
        let mut status = Status::new(false, 0);
        let result = status.load_talk_model(0);
        assert_eq!(Ok(()), result);
        assert_eq!(1, status.talk_models.predict_duration.len());
        assert_eq!(1, status.talk_models.predict_intonation.len());
        assert_eq!(1, status.talk_models.decode.len());
    }

    #[rstest]
    fn status_is_talk_model_loaded_works() {
        let mut status = Status::new(false, 0);
        let model_index = 0;
        assert!(
            !status.is_talk_model_loaded(model_index),
            "model should  not be loaded"
        );
        let result = status.load_talk_model(model_index);
        assert_eq!(Ok(()), result);
        assert!(
            status.is_talk_model_loaded(model_index),
            "model should be loaded"
        );
    }

    #[rstest]
    fn status_load_sing_teacher_model_works() {
        let mut status = Status::new(false, 0);
        let result = status.load_sing_teacher_model(0);
        assert_eq!(Ok(()), result);
        assert_eq!(
            1,
            status
                .sing_teacher_models
                .predict_sing_consonant_length
                .len()
        );
        assert_eq!(1, status.sing_teacher_models.predict_sing_f0.len());
        assert_eq!(1, status.sing_teacher_models.predict_sing_volume.len());
    }

    #[rstest]
    fn status_is_sing_teacher_model_loaded_works() {
        let mut status = Status::new(false, 0);
        let model_index = 0;
        assert!(
            !status.is_sing_teacher_model_loaded(model_index),
            "model should  not be loaded"
        );
        let result = status.load_sing_teacher_model(model_index);
        assert_eq!(Ok(()), result);
        assert!(
            status.is_sing_teacher_model_loaded(model_index),
            "model should be loaded"
        );
    }

    #[rstest]
    fn status_load_sf_decode_model_works() {
        let mut status = Status::new(false, 0);
        let result = status.load_sf_decode_model(0);
        assert_eq!(Ok(()), result);
        assert_eq!(1, status.sf_decode_models.sf_decode.len());
    }

    #[rstest]
    fn status_is_sf_decode_model_loaded_works() {
        let mut status = Status::new(false, 0);
        let model_index = 0;
        assert!(
            !status.is_sf_decode_model_loaded(model_index),
            "model should  not be loaded"
        );
        let result = status.load_sf_decode_model(model_index);
        assert_eq!(Ok(()), result);
        assert!(
            status.is_sf_decode_model_loaded(model_index),
            "model should be loaded"
        );
    }
}
