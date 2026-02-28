use std::{
    any,
    fmt::{self, Debug},
};

use duplicate::{duplicate, duplicate_item};
use educe::Educe;
use enum_map::EnumMap;
use indexmap::IndexMap;
use itertools::iproduct;

use crate::{
    error::{ErrorRepr, LoadModelError, LoadModelErrorKind, LoadModelResult},
    Result,
};

use super::{
    infer::{
        self,
        domains::{
            inference_domain_map_values, ExperimentalTalkDomain, FrameDecodeDomain,
            InferenceDomainMap, SingingTeacherDomain, TalkDomain,
        },
        session_set::{InferenceSessionCell, InferenceSessionSet},
        InferenceDomain, InferenceInputSignature, InferenceRuntime, InferenceSessionOptions,
        InferenceSignature,
    },
    manifest::{InnerVoiceId, StyleIdToInnerVoiceId},
    metas::{self, CharacterMeta, StyleId, StyleMeta, VoiceModelMeta},
    voice_model::{ModelBytesWithInnerVoiceIdsByDomain, VoiceModelHeader, VoiceModelId},
};

#[derive(Debug)]
pub(crate) struct Status<R: InferenceRuntime> {
    pub(crate) rt: &'static R,
    loaded_models: std::sync::Mutex<LoadedModels<R>>,
    session_options: InferenceDomainMap<SessionOptionsByDomain>,
}

impl<R: InferenceRuntime> Status<R> {
    pub(crate) fn new(
        rt: &'static R,
        session_options: InferenceDomainMap<SessionOptionsByDomain>,
    ) -> Self {
        Self {
            rt,
            loaded_models: Default::default(),
            session_options,
        }
    }

    pub(crate) fn insert_model(
        &self,
        model_header: &VoiceModelHeader,
        model_contents: &InferenceDomainMap<ModelBytesWithInnerVoiceIdsByDomain>,
    ) -> Result<()> {
        self.loaded_models
            .lock()
            .unwrap()
            .ensure_acceptable(model_header)?;

        let session_sets_with_inner_ids = model_contents
            .create_session_sets(self.rt, &self.session_options)
            .map_err(|source| LoadModelError {
                path: model_header.path.clone(),
                context: LoadModelErrorKind::InvalidModelData,
                source: Some(source),
            })?;

        self.loaded_models
            .lock()
            .unwrap()
            .insert(model_header, session_sets_with_inner_ids)?;
        Ok(())
    }

    pub(crate) fn unload_model(&self, voice_model_id: VoiceModelId) -> Result<()> {
        self.loaded_models.lock().unwrap().remove(voice_model_id)
    }

    pub(crate) fn metas(&self) -> VoiceModelMeta {
        self.loaded_models.lock().unwrap().metas()
    }

    /// あるスタイルに対応する`VoiceModelId`と`InnerVoiceId`の組を返す。
    ///
    /// `StyleId` → `InnerVoiceId`のマッピングが存在しない場合は、`InnerVoiceId`としては
    /// `style_id`と同じ値を返す。
    pub(crate) fn ids_for<D: InferenceDomainExt>(
        &self,
        style_id: StyleId,
    ) -> Result<(VoiceModelId, InnerVoiceId)> {
        self.loaded_models.lock().unwrap().ids_for::<D>(style_id)
    }

    pub(crate) fn contains_domain<D: InferenceDomainExt>(&self, style_id: StyleId) -> bool {
        self.loaded_models
            .lock()
            .unwrap_or_else(|e| panic!("{e}"))
            .contains_domain::<D>(style_id)
    }

    pub(crate) fn is_loaded_model(&self, voice_model_id: VoiceModelId) -> bool {
        self.loaded_models
            .lock()
            .unwrap()
            .contains_voice_model(voice_model_id)
    }

    // FIXME: この関数はcompatible_engineとテストでのみ使われるが、テストのために`StyleType`を
    // 引数に含めるようにする
    pub(crate) fn is_loaded_model_by_style_id(&self, style_id: StyleId) -> bool {
        self.loaded_models.lock().unwrap().contains_style(style_id)
    }

    /// 推論を実行する。
    ///
    /// # Performance
    ///
    /// CPU/GPU-boundな操作であるため、非同期ランタイム上では直接実行されるべきではない。
    ///
    /// # Panics
    ///
    /// `self`が`model_id`を含んでいないとき、パニックする。
    pub(crate) async fn run_session<A, I>(
        &self,
        model_id: VoiceModelId,
        input: I,
        cancellable: A::Cancellable,
    ) -> Result<<I::Signature as InferenceSignature>::Output>
    where
        A: infer::AsyncExt,
        I: InferenceInputSignature,
        <I::Signature as InferenceSignature>::Domain: InferenceDomainExt,
    {
        let sess = self.loaded_models.lock().unwrap().get(model_id);
        sess.run::<A>(input, cancellable).await
    }
}

/// 読み込んだモデルの`Session`とそのメタ情報を保有し、追加/削除/取得の操作を提供する。
///
/// この構造体のメソッドは、すべて一瞬で完了すべきである。
#[derive(Educe)]
#[educe(Default(bound = "R: InferenceRuntime"))]
struct LoadedModels<R: InferenceRuntime>(IndexMap<VoiceModelId, LoadedModel<R>>);

impl<R: InferenceRuntime> Debug for LoadedModels<R> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_map()
            .entries(self.0.keys().map(|id| (id, format_args!("_"))))
            .finish()
    }
}

struct LoadedModel<R: InferenceRuntime> {
    metas: VoiceModelMeta,
    session_sets_with_inner_ids: InferenceDomainMap<SessionSetsWithInnerVoiceIdsByDomain<R>>,
}

impl<R: InferenceRuntime> LoadedModels<R> {
    fn metas(&self) -> VoiceModelMeta {
        metas::merge(self.0.values().flat_map(|LoadedModel { metas, .. }| metas))
    }

    fn ids_for<D: InferenceDomainExt>(
        &self,
        style_id: StyleId,
    ) -> Result<(VoiceModelId, InnerVoiceId)> {
        let (
            model_id,
            LoadedModel {
                session_sets_with_inner_ids,
                ..
            },
        ) = self
            .0
            .iter()
            .filter(
                |(
                    _,
                    LoadedModel {
                        session_sets_with_inner_ids,
                        ..
                    },
                )| D::visit(session_sets_with_inner_ids).is_some(),
            )
            .find(|(_, LoadedModel { metas, .. })| {
                metas
                    .iter()
                    .flat_map(|CharacterMeta { styles, .. }| styles)
                    .any(|style| style.id == style_id && D::style_types().contains(&style.r#type))
            })
            .ok_or(ErrorRepr::StyleNotFound {
                style_id,
                style_types: D::style_types(),
            })?;

        let inner_voice_id = session_sets_with_inner_ids
            .get::<D>()
            .as_ref()
            .and_then(|(inner_voice_ids, _)| inner_voice_ids.get(&style_id).copied())
            .unwrap_or_else(|| InnerVoiceId::new(style_id.0));

        Ok((*model_id, inner_voice_id))
    }

    /// # Panics
    ///
    /// 次の場合にパニックする。
    ///
    /// - `self`が`model_id`を含んでいないとき
    /// - 対応する`InferenceDomain`が欠けているとき
    fn get<I>(&self, model_id: VoiceModelId) -> InferenceSessionCell<R, I>
    where
        I: InferenceInputSignature,
        <I::Signature as InferenceSignature>::Domain: InferenceDomainExt,
    {
        let (_, session_set) = self.0[&model_id]
            .session_sets_with_inner_ids
            .get::<<I::Signature as InferenceSignature>::Domain>()
            .as_ref()
            .unwrap_or_else(|| {
                let type_name = any::type_name::<<I::Signature as InferenceSignature>::Domain>()
                    .split("::")
                    .last()
                    .unwrap();
                panic!(
                    "missing session set for `{type_name}` (should be checked in \
                     `VoiceModelHeader::new` and `ids_for`)",
                );
            });
        session_set.get()
    }

    fn contains_domain<D: InferenceDomainExt>(&self, style_id: StyleId) -> bool {
        self.0
            .iter()
            .find(|(_, LoadedModel { metas, .. })| {
                metas
                    .iter()
                    .flat_map(|CharacterMeta { styles, .. }| styles)
                    .any(|style| style.id == style_id && D::style_types().contains(&style.r#type))
            })
            .and_then(
                |(
                    _,
                    LoadedModel {
                        session_sets_with_inner_ids,
                        ..
                    },
                )| session_sets_with_inner_ids.get::<D>(),
            )
            .is_some()
    }

    fn contains_voice_model(&self, model_id: VoiceModelId) -> bool {
        self.0.contains_key(&model_id)
    }

    fn contains_style(&self, style_id: StyleId) -> bool {
        self.styles().any(|style| style.id == style_id)
    }

    /// 音声モデルを受け入れ可能かをチェックする。
    ///
    /// # Errors
    ///
    /// 次の場合にエラーを返す。
    ///
    /// - 現在持っている音声モデルIDかスタイルIDが`model_header`と重複するとき
    /// - 必要であるはずの`InferenceDomain`のモデルデータが欠けているとき
    // FIXME: コメントとテストを書く
    // - https://github.com/VOICEVOX/voicevox_core/pull/761#discussion_r1589978521
    // - https://github.com/VOICEVOX/voicevox_core/pull/761#discussion_r1589976759
    fn ensure_acceptable(&self, model_header: &VoiceModelHeader) -> LoadModelResult<()> {
        let error = |context| LoadModelError {
            path: model_header.path.clone(),
            context,
            source: None,
        };

        if self.0.contains_key(&model_header.manifest.id) {
            return Err(error(LoadModelErrorKind::ModelAlreadyLoaded {
                id: model_header.manifest.id,
            }));
        }

        // FIXME: https://github.com/VOICEVOX/voicevox_core/pull/761#discussion_r1590200343

        let loaded = self.characters();
        let external = model_header.metas.iter();
        for (loaded, external) in iproduct!(loaded, external) {
            if loaded.speaker_uuid == external.speaker_uuid {
                loaded.warn_diff_except_styles(external);
            }
        }

        let loaded = self.styles().map(|&StyleMeta { id, .. }| id);
        let external = model_header
            .metas
            .iter()
            .flat_map(|CharacterMeta { styles, .. }| styles)
            .map(|&StyleMeta { id, .. }| id);
        if let Some((id, _)) =
            iproduct!(loaded, external).find(|(loaded, external)| loaded == external)
        {
            return Err(error(LoadModelErrorKind::StyleAlreadyLoaded { id }));
        }
        Ok(())
    }

    fn insert(
        &mut self,
        model_header: &VoiceModelHeader,
        session_sets_with_inner_ids: InferenceDomainMap<SessionSetsWithInnerVoiceIdsByDomain<R>>,
    ) -> Result<()> {
        self.ensure_acceptable(model_header)?;

        let prev = self.0.insert(
            model_header.manifest.id,
            LoadedModel {
                metas: model_header.metas.clone(),
                session_sets_with_inner_ids,
            },
        );
        assert!(prev.is_none());
        Ok(())
    }

    fn remove(&mut self, model_id: VoiceModelId) -> Result<()> {
        if self.0.shift_remove(&model_id).is_none() {
            return Err(ErrorRepr::ModelNotFound { model_id }.into());
        }
        Ok(())
    }

    fn characters(&self) -> impl Iterator<Item = &CharacterMeta> + Clone {
        self.0.values().flat_map(|LoadedModel { metas, .. }| metas)
    }

    fn styles(&self) -> impl Iterator<Item = &StyleMeta> {
        self.characters()
            .flat_map(|CharacterMeta { styles, .. }| styles)
    }
}

pub(crate) trait InferenceDomainExt: InferenceDomain {
    fn visit<R: InferenceRuntime>(
        map: &InferenceDomainMap<SessionSetsWithInnerVoiceIdsByDomain<R>>,
    ) -> Option<&(StyleIdToInnerVoiceId, InferenceSessionSet<R, Self>)>;
}

#[duplicate_item(
    T                        field;
    [ TalkDomain ]           [ talk ];
    [ ExperimentalTalkDomain ] [ experimental_talk ];
    [ SingingTeacherDomain ] [ singing_teacher ];
    [ FrameDecodeDomain ]    [ frame_decode ];
)]
impl InferenceDomainExt for T {
    fn visit<R: InferenceRuntime>(
        map: &InferenceDomainMap<SessionSetsWithInnerVoiceIdsByDomain<R>>,
    ) -> Option<&(StyleIdToInnerVoiceId, InferenceSessionSet<R, Self>)> {
        map.field.as_ref()
    }
}

impl<R: InferenceRuntime> InferenceDomainMap<SessionSetsWithInnerVoiceIdsByDomain<R>> {
    fn get<D: InferenceDomainExt>(
        &self,
    ) -> Option<&(StyleIdToInnerVoiceId, InferenceSessionSet<R, D>)> {
        D::visit(self)
    }
}

impl InferenceDomainMap<ModelBytesWithInnerVoiceIdsByDomain> {
    fn create_session_sets<R: InferenceRuntime>(
        &self,
        rt: &R,
        session_options: &InferenceDomainMap<SessionOptionsByDomain>,
    ) -> anyhow::Result<InferenceDomainMap<SessionSetsWithInnerVoiceIdsByDomain<R>>> {
        duplicate! {
            [
                field;
                [ talk ];
                [ experimental_talk ];
                [ singing_teacher ];
                [ frame_decode ];
            ]
            let field = self
                .field
                .as_ref()
                .map(|(inner_voice_ids, model_bytes)| {
                    let session_set = InferenceSessionSet::new(rt, model_bytes, &session_options.field)?;
                    Ok::<_, anyhow::Error>((inner_voice_ids.clone(), session_set))
                })
                .transpose()?;
        }

        Ok(InferenceDomainMap {
            talk,
            experimental_talk,
            singing_teacher,
            frame_decode,
        })
    }
}

type SessionOptionsByDomain =
    inference_domain_map_values!(for<D> EnumMap<D::Operation, InferenceSessionOptions>);

type SessionSetsWithInnerVoiceIdsByDomain<R> =
    inference_domain_map_values!(for<D> Option<(StyleIdToInnerVoiceId, InferenceSessionSet<R, D>)>);

#[cfg(test)]
mod tests {
    use enum_map::enum_map;
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use crate::macros::tests::assert_debug_fmt_eq;

    use super::{
        super::{
            devices::{DeviceSpec, GpuSpec},
            infer::{
                domains::{
                    ExperimentalTalkOperation, FrameDecodeOperation, InferenceDomainMap,
                    SingingTeacherOperation, TalkOperation,
                },
                InferenceSessionOptions,
            },
        },
        Status,
    };

    #[rstest]
    #[case(DeviceSpec::Gpu(GpuSpec::Cuda), 0)]
    #[case(DeviceSpec::Gpu(GpuSpec::Cuda), 1)]
    #[case(DeviceSpec::Gpu(GpuSpec::Cuda), 8)]
    #[case(DeviceSpec::Cpu, 2)]
    #[case(DeviceSpec::Cpu, 4)]
    #[case(DeviceSpec::Cpu, 8)]
    #[case(DeviceSpec::Cpu, 0)]
    fn status_new_works(#[case] device_for_heavy: DeviceSpec, #[case] cpu_num_threads: u16) {
        let light_session_options = InferenceSessionOptions::new(cpu_num_threads, DeviceSpec::Cpu);
        let heavy_session_options = InferenceSessionOptions::new(cpu_num_threads, device_for_heavy);
        let session_options = InferenceDomainMap {
            talk: enum_map! {
                TalkOperation::PredictDuration | TalkOperation::PredictIntonation => {
                    light_session_options
                }
                TalkOperation::Decode => heavy_session_options,
            },
            experimental_talk: enum_map! {
                ExperimentalTalkOperation::PredictDuration
                | ExperimentalTalkOperation::PredictIntonation
                | ExperimentalTalkOperation::GenerateFullIntermediate => light_session_options,
                ExperimentalTalkOperation::RenderAudioSegment => heavy_session_options,
            },
            singing_teacher: enum_map! {
                SingingTeacherOperation::PredictSingConsonantLength
                | SingingTeacherOperation::PredictSingF0
                | SingingTeacherOperation::PredictSingVolume => light_session_options,
            },
            frame_decode: enum_map! {
                FrameDecodeOperation::SfDecode => heavy_session_options,
            },
        };
        let status = Status::new(
            crate::blocking::Onnxruntime::from_test_util_data().unwrap(),
            session_options,
        );

        assert_eq!(
            light_session_options,
            status.session_options.experimental_talk[ExperimentalTalkOperation::PredictDuration],
        );
        assert_eq!(
            light_session_options,
            status.session_options.experimental_talk[ExperimentalTalkOperation::PredictIntonation],
        );
        assert_eq!(
            light_session_options,
            status.session_options.experimental_talk
                [ExperimentalTalkOperation::GenerateFullIntermediate],
        );
        assert_eq!(
            heavy_session_options,
            status.session_options.experimental_talk[ExperimentalTalkOperation::RenderAudioSegment],
        );

        assert!(status.loaded_models.lock().unwrap().0.is_empty());
    }

    #[rstest]
    #[tokio::test]
    async fn status_load_model_works() {
        let status = Status::new(
            crate::blocking::Onnxruntime::from_test_util_data().unwrap(),
            InferenceDomainMap {
                talk: enum_map!(_ => InferenceSessionOptions::new(0, DeviceSpec::Cpu)),
                experimental_talk: enum_map!(_ => InferenceSessionOptions::new(0, DeviceSpec::Cpu)),
                singing_teacher: enum_map!(_ => InferenceSessionOptions::new(0, DeviceSpec::Cpu)),
                frame_decode: enum_map!(_ => InferenceSessionOptions::new(0, DeviceSpec::Cpu)),
            },
        );
        let model = &crate::nonblocking::VoiceModelFile::sample().await.unwrap();
        let model_contents = &model.inner().read_inference_models().await.unwrap();
        let result = status.insert_model(model.inner().header(), model_contents);
        assert_debug_fmt_eq!(Ok(()), result);
        assert_eq!(1, status.loaded_models.lock().unwrap().0.len());
    }

    #[rstest]
    #[tokio::test]
    async fn status_is_model_loaded_works() {
        let status = Status::new(
            crate::blocking::Onnxruntime::from_test_util_data().unwrap(),
            InferenceDomainMap {
                talk: enum_map!(_ => InferenceSessionOptions::new(0, DeviceSpec::Cpu)),
                experimental_talk: enum_map!(_ => InferenceSessionOptions::new(0, DeviceSpec::Cpu)),
                singing_teacher: enum_map!(_ => InferenceSessionOptions::new(0, DeviceSpec::Cpu)),
                frame_decode: enum_map!(_ => InferenceSessionOptions::new(0, DeviceSpec::Cpu)),
            },
        );
        let vvm = &crate::nonblocking::VoiceModelFile::sample().await.unwrap();
        let model_header = vvm.inner().header();
        let model_contents = &vvm.inner().read_inference_models().await.unwrap();
        assert!(
            !status.is_loaded_model(model_header.manifest.id),
            "model should  not be loaded"
        );
        let result = status.insert_model(model_header, model_contents);
        assert_debug_fmt_eq!(Ok(()), result);
        assert!(
            status.is_loaded_model(model_header.manifest.id),
            "model should be loaded",
        );
    }
}
