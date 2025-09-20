//! 音声シンセサイザ。
//!
//! メインの部分。[`crate::core`]と[`crate::engine`]の二つはここで用いる。

use easy_ext::ext;
use enum_map::enum_map;
use futures_util::TryFutureExt as _;
use std::{
    fmt::{self, Debug},
    future::Future,
    marker::PhantomData,
    ops::Range,
    sync::Arc,
};
use tracing::info;

use crate::{
    asyncs::{Async, BlockingThreadPool, SingleTasked},
    core::{
        devices::{self, DeviceSpec, GpuSpec},
        ensure_minimum_phoneme_length,
        infer::{
            self,
            domains::{
                experimental_talk, talk, DecodeInput, DecodeOutput, ExperimentalTalkDomain,
                ExperimentalTalkOperation, FrameDecodeDomain, FrameDecodeOperation,
                GenerateFullIntermediateInput, GenerateFullIntermediateOutput, InferenceDomainMap,
                PredictSingConsonantLengthInput, PredictSingConsonantLengthOutput,
                PredictSingF0Input, PredictSingF0Output, PredictSingVolumeInput,
                PredictSingVolumeOutput, RenderAudioSegmentInput, RenderAudioSegmentOutput,
                SfDecodeInput, SfDecodeOutput, SingingTeacherDomain, SingingTeacherOperation,
                TalkDomain, TalkOperation,
            },
            InferenceRuntime, InferenceSessionOptions,
        },
        pad_decoder_feature,
        status::Status,
        voice_model,
        voice_spec::{VoiceSpec, VoiceSpecExt as _},
    },
    engine::{
        talk::{create_kana, initial_process, parse_kana, split_mora, DecoderFeature, Mora},
        to_s16le_pcm, wav_from_s16le, OjtPhoneme,
    },
    error::ErrorRepr,
    future::FutureExt as _,
    AccentPhrase, AudioQuery, Result, StyleId, VoiceModelId, VoiceModelMeta,
};

pub const DEFAULT_CPU_NUM_THREADS: u16 = 0;
pub const DEFAULT_ENABLE_INTERROGATIVE_UPSPEAK: bool = true;
pub const DEFAULT_HEAVY_INFERENCE_CANCELLABLE: bool =
    <BlockingThreadPool as infer::AsyncExt>::DEFAULT_HEAVY_INFERENCE_CANCELLABLE;

#[derive(derive_more::Debug)]
#[debug(bound(A::Cancellable: Debug))]
struct SynthesisOptions<A: infer::AsyncExt> {
    enable_interrogative_upspeak: bool,
    cancellable: A::Cancellable,
}

impl<A: infer::AsyncExt> Default for SynthesisOptions<A> {
    fn default() -> Self {
        Self {
            enable_interrogative_upspeak: DEFAULT_ENABLE_INTERROGATIVE_UPSPEAK,
            cancellable: A::DEFAULT_HEAVY_INFERENCE_CANCELLABLE,
        }
    }
}

// FIXME: this is dead code
impl<A: infer::AsyncExt> AsRef<SynthesisOptions<A>> for SynthesisOptions<A> {
    fn as_ref(&self) -> &SynthesisOptions<A> {
        self
    }
}

impl<A: infer::AsyncExt> From<&TtsOptions<A>> for SynthesisOptions<A> {
    fn from(options: &TtsOptions<A>) -> Self {
        Self {
            enable_interrogative_upspeak: options.enable_interrogative_upspeak,
            cancellable: options.cancellable,
        }
    }
}

#[derive(derive_more::Debug)]
#[debug(bound(A::Cancellable: Debug))]
struct TtsOptions<A: infer::AsyncExt> {
    enable_interrogative_upspeak: bool,
    cancellable: A::Cancellable,
}

impl<A: infer::AsyncExt> Default for TtsOptions<A> {
    fn default() -> Self {
        Self {
            enable_interrogative_upspeak: DEFAULT_ENABLE_INTERROGATIVE_UPSPEAK,
            cancellable: A::DEFAULT_HEAVY_INFERENCE_CANCELLABLE,
        }
    }
}

/// ハードウェアアクセラレーションモードを設定する設定値。
#[cfg_attr(doc, doc(alias = "VoicevoxAccelerationMode"))]
#[expect(
    clippy::manual_non_exhaustive,
    reason = "バインディングを作るときはexhaustiveとして扱いたい"
)]
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AccelerationMode {
    /// 実行環境に合った適切なハードウェアアクセラレーションモードを選択する。
    #[default]
    Auto,
    /// ハードウェアアクセラレーションモードを"CPU"に設定する。
    Cpu,
    /// ハードウェアアクセラレーションモードを"GPU"に設定する。
    Gpu,
    #[doc(hidden)]
    __NonExhaustive,
}

#[derive(Debug)]
struct InitializeOptions {
    acceleration_mode: AccelerationMode,
    cpu_num_threads: u16,
}

impl Default for InitializeOptions {
    fn default() -> Self {
        Self {
            acceleration_mode: Default::default(),
            cpu_num_threads: DEFAULT_CPU_NUM_THREADS,
        }
    }
}

trait AsyncExt: infer::AsyncExt {
    async fn unblock<T, F>(f: F) -> T
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static;
}

impl AsyncExt for SingleTasked {
    async fn unblock<T, F>(f: F) -> T
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        f()
    }
}

impl AsyncExt for BlockingThreadPool {
    async fn unblock<T, F>(f: F) -> T
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        ::blocking::unblock(f).await
    }
}

const DEFAULT_SAMPLING_RATE: u32 = 24000;
/// 音が途切れてしまうのを避けるworkaround処理のためのパディング幅（フレーム数）
const PADDING_FRAME_LENGTH: usize = 38; // (0.4秒 * 24000Hz / 256.0).round()
/// 音声生成の際、音声特徴量の前後に確保すべきマージン幅（フレーム数）
/// モデルの受容野から計算される
pub const MARGIN: usize = 14;
/// 指定した音声区間に対応する特徴量を両端にマージンを追加した上で切り出す
fn crop_with_margin(audio: &AudioFeature, range: Range<usize>) -> ndarray::ArrayView2<'_, f32> {
    if range.start > audio.frame_length || range.end > audio.frame_length {
        panic!(
            "{range:?} is out of range for audio feature of length {frame_length}",
            frame_length = audio.frame_length,
        );
    }
    if range.start > range.end {
        panic!("{range:?} is invalid because start > end",);
    }
    let range = range.start..range.end + 2 * MARGIN;
    audio.internal_state.slice(ndarray::s![range, ..])
}
/// 追加した安全マージンを生成音声から取り除く
fn trim_margin_from_wave(wave_with_margin: ndarray::Array1<f32>) -> ndarray::Array1<f32> {
    let len = wave_with_margin.len();
    wave_with_margin.slice_move(ndarray::s![MARGIN * 256..len - MARGIN * 256])
}

/// 音声の中間表現。
// TODO: 後で復活させる
// https://github.com/VOICEVOX/voicevox_core/issues/970
#[doc(hidden)]
#[derive(Clone, PartialEq, derive_more::Debug)]
pub struct AudioFeature {
    /// (フレーム数, 特徴数)の形を持つ音声特徴量。
    #[debug("_")]
    internal_state: ndarray::Array2<f32>,
    /// 生成時に指定したスタイル番号。
    style_id: crate::StyleId,
    /// workaround paddingを除いた音声特徴量のフレーム数。
    pub frame_length: usize,
    /// フレームレート。全体の秒数は`frame_length / frame_rate`で表せる。
    pub frame_rate: f64,
    /// 生成時に利用したクエリ。
    audio_query: AudioQuery,
}

#[derive(derive_more::Debug)]
struct Inner<T, A: Async> {
    status: Arc<Status<crate::blocking::Onnxruntime>>,
    text_analyzer: T,
    use_gpu: bool,
    #[debug(ignore)]
    _marker: PhantomData<fn(A) -> A>,
}

#[derive(derive_more::Debug)]
struct InnerRefWithoutTextAnalyzer<'a, A: Async> {
    status: &'a Arc<Status<crate::blocking::Onnxruntime>>,
    use_gpu: bool,
    #[debug(ignore)]
    _marker: PhantomData<fn(A) -> A>,
}

impl<T> From<Inner<T, BlockingThreadPool>>
    for Inner<AssumeSingleTasked<AssumeBlockable<T>>, SingleTasked>
{
    fn from(from: Inner<T, BlockingThreadPool>) -> Self {
        Self {
            status: from.status,
            text_analyzer: AssumeSingleTasked(AssumeBlockable(from.text_analyzer)),
            use_gpu: from.use_gpu,
            _marker: PhantomData,
        }
    }
}

#[derive(derive_more::Debug)]
#[debug("{_0:?}")]
struct AssumeSingleTasked<T>(T);

impl<T: crate::blocking::TextAnalyzer> crate::nonblocking::TextAnalyzer for AssumeSingleTasked<T> {
    async fn analyze(&self, text: &str) -> anyhow::Result<Vec<AccentPhrase>> {
        self.0.analyze(text)
    }
}

pub struct AssumeBlockable<T>(T);

impl<T: crate::nonblocking::TextAnalyzer> crate::blocking::TextAnalyzer for AssumeBlockable<T> {
    fn analyze(&self, text: &str) -> anyhow::Result<Vec<AccentPhrase>> {
        self.0.analyze(text).block_on()
    }
}

impl<T, A: AsyncExt> Inner<T, A> {
    fn new(
        onnxruntime: &'static crate::blocking::Onnxruntime,
        text_analyzer: T,
        options: &InitializeOptions,
    ) -> Result<Self> {
        #[cfg(windows)]
        list_windows_video_cards();

        let test_gpus = || {
            info!("GPUをテストします:");
            let availabilities = devices::test_gpus(
                GpuSpec::defaults(),
                crate::blocking::Onnxruntime::DISPLAY_NAME,
                onnxruntime.supported_devices()?,
                |gpu| onnxruntime.test_gpu(gpu),
            );
            for line in availabilities.to_string().lines() {
                info!("  {line}");
            }
            crate::Result::Ok(availabilities)
        };

        let device_for_heavy = match options.acceleration_mode {
            AccelerationMode::Auto => match *test_gpus()?.oks() {
                [] => DeviceSpec::Cpu,
                [gpu, ..] => DeviceSpec::Gpu(gpu),
            },
            AccelerationMode::Cpu => DeviceSpec::Cpu,
            AccelerationMode::Gpu => {
                let availabilities = test_gpus()?;
                match *availabilities.oks() {
                    [] => return Err(ErrorRepr::GpuSupport(availabilities).into()),
                    [gpu, ..] => DeviceSpec::Gpu(gpu),
                }
            }
            AccelerationMode::__NonExhaustive => unreachable!(),
        };

        info!("{device_for_heavy}を利用します");

        // 軽いモデルはこちらを使う
        let light_session_options =
            InferenceSessionOptions::new(options.cpu_num_threads, DeviceSpec::Cpu);

        // 重いモデルはこちらを使う
        let heavy_session_options =
            InferenceSessionOptions::new(options.cpu_num_threads, device_for_heavy);

        let status = Status::new(
            onnxruntime,
            InferenceDomainMap {
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
            },
        )
        .into();

        let use_gpu = matches!(device_for_heavy, DeviceSpec::Gpu(_));

        Ok(Self {
            status,
            text_analyzer,
            use_gpu,
            _marker: PhantomData,
        })
    }

    fn without_text_analyzer(&self) -> InnerRefWithoutTextAnalyzer<'_, A> {
        InnerRefWithoutTextAnalyzer {
            status: &self.status,
            use_gpu: self.use_gpu,
            _marker: PhantomData,
        }
    }

    fn fill_debug_struct_body(&self, mut fmt: fmt::DebugStruct<'_, '_>) -> fmt::Result
    where
        T: Debug,
    {
        let Self {
            status,
            text_analyzer,
            use_gpu,
            _marker: _,
        } = self;

        fmt.field("status", status)
            .field("text_analyzer", text_analyzer)
            .field("use_gpu", use_gpu)
            .finish_non_exhaustive()
    }
}

trait AsInner {
    type TextAnalyzer;
    type Async: AsyncExt;
    fn status(&self) -> &Arc<Status<crate::blocking::Onnxruntime>>;
    fn text_analyzer(&self) -> &Self::TextAnalyzer;
    fn use_gpu(&self) -> bool;

    fn onnxruntime(&self) -> &'static crate::blocking::Onnxruntime {
        self.status().rt
    }

    fn is_gpu_mode(&self) -> bool {
        self.use_gpu()
    }

    async fn load_voice_model(&self, model: &voice_model::Inner<Self::Async>) -> crate::Result<()> {
        let model_bytes = model.read_inference_models().await?;

        let status = self.status().clone();
        let header = model.header().clone();
        Self::Async::unblock(move || status.insert_model(&header, &model_bytes)).await
    }

    fn unload_voice_model(&self, voice_model_id: VoiceModelId) -> Result<()> {
        self.status().unload_model(voice_model_id)
    }

    fn is_loaded_voice_model(&self, voice_model_id: VoiceModelId) -> bool {
        self.status().is_loaded_model(voice_model_id)
    }

    fn is_loaded_model_by_style_id(&self, style_id: StyleId) -> bool {
        self.status().is_loaded_model_by_style_id(style_id)
    }

    fn metas(&self) -> VoiceModelMeta {
        self.status().metas()
    }

    async fn precompute_render(
        &self,
        audio_query: &AudioQuery,
        voice_spec: impl VoiceSpec,
        options: &SynthesisOptions<Self::Async>,
    ) -> Result<AudioFeature> {
        let (_, style_id) = self.status().find_voice(voice_spec)?; // TODO: `generate_full_intermediate`が`StyleId`を返すようにする
        let DecoderFeature { f0, phoneme } =
            audio_query.decoder_feature(options.enable_interrogative_upspeak);

        let spec = self
            .generate_full_intermediate(
                f0.len(),
                OjtPhoneme::num_phoneme(),
                &f0,
                phoneme.as_flattened(),
                style_id,
            )
            .await?;
        Ok(AudioFeature {
            internal_state: spec,
            style_id,
            frame_length: f0.len(),
            frame_rate: (DEFAULT_SAMPLING_RATE as f64) / 256.0,
            audio_query: audio_query.clone(),
        })
    }

    async fn render(&self, audio: &AudioFeature, range: Range<usize>) -> Result<Vec<u8>> {
        // TODO: 44.1kHzなどの対応
        if range.is_empty() {
            // FIXME: `start>end`に対してパニックせずに正常に空を返してしまうのでは？
            // 指定区間が空のときは早期リターン
            return Ok(vec![]);
        }
        let spec_segment = crop_with_margin(audio, range);
        let wave_with_margin = self
            .render_audio_segment(spec_segment.to_owned(), audio.style_id)
            .await?;
        let wave = trim_margin_from_wave(wave_with_margin);
        Ok(to_s16le_pcm::<DEFAULT_SAMPLING_RATE>(
            wave.as_slice()
                .expect("`trim_margin_from_wave` should just trim an array"),
            &audio.audio_query,
        ))
    }

    async fn synthesis(
        &self,
        audio_query: &AudioQuery,
        voice_spec: impl VoiceSpec,
        options: &SynthesisOptions<Self::Async>,
    ) -> Result<Vec<u8>> {
        let (_, style_id) = self.status().find_voice(voice_spec)?; // TODO: `generate_full_intermediate`が`StyleId`を返すようにする
        if self.status().contains_domain::<TalkDomain>(style_id) {
            let DecoderFeature { f0, phoneme } =
                audio_query.decoder_feature(options.enable_interrogative_upspeak);
            let wave = &self
                .decode(
                    f0.len(),
                    OjtPhoneme::num_phoneme(),
                    &f0,
                    phoneme.as_flattened(),
                    style_id,
                    options.cancellable,
                )
                .await?;
            return Ok(wav_from_s16le(
                &to_s16le_pcm::<DEFAULT_SAMPLING_RATE>(wave, audio_query),
                audio_query.output_sampling_rate,
                audio_query.output_stereo,
            ));
        }
        let audio = self
            .precompute_render(audio_query, style_id, options)
            .await?;
        let pcm = self.render(&audio, 0..audio.frame_length).await?;
        Ok(wav_from_s16le(
            &pcm,
            audio_query.output_sampling_rate,
            audio_query.output_stereo,
        ))
    }

    async fn create_accent_phrases_from_kana(
        &self,
        kana: &str,
        voice_spec: impl VoiceSpec,
    ) -> Result<Vec<AccentPhrase>> {
        let accent_phrases = parse_kana(kana)?;
        self.replace_mora_data(&accent_phrases, voice_spec).await
    }

    async fn replace_mora_data(
        &self,
        accent_phrases: &[AccentPhrase],
        mut voice_spec: impl VoiceSpec,
    ) -> Result<Vec<AccentPhrase>> {
        let accent_phrases = self
            .replace_phoneme_length(accent_phrases, voice_spec.as_mut())
            .await?;
        self.replace_mora_pitch(&accent_phrases, voice_spec).await
    }

    async fn replace_phoneme_length(
        &self,
        accent_phrases: &[AccentPhrase],
        voice_spec: impl VoiceSpec,
    ) -> Result<Vec<AccentPhrase>> {
        let (_, phoneme_data_list) = initial_process(accent_phrases);

        let (_, _, vowel_indexes_data) = split_mora(&phoneme_data_list);

        let phoneme_list_s: Vec<i64> = phoneme_data_list
            .iter()
            .map(|phoneme_data| phoneme_data.phoneme_id())
            .collect();
        let phoneme_length = self.predict_duration(&phoneme_list_s, voice_spec).await?;

        let mut index = 0;
        let new_accent_phrases = accent_phrases
            .iter()
            .map(|accent_phrase| AccentPhrase {
                moras: accent_phrase
                    .moras
                    .iter()
                    .map(|mora| {
                        let new_mora = Mora {
                            consonant_length: mora.consonant.as_ref().map(|_| {
                                phoneme_length[vowel_indexes_data[index + 1] as usize - 1]
                            }),
                            vowel_length: phoneme_length[vowel_indexes_data[index + 1] as usize],
                            ..mora.clone()
                        };
                        index += 1;
                        new_mora
                    })
                    .collect(),
                pause_mora: accent_phrase.pause_mora.as_ref().map(|pause_mora| {
                    let new_pause_mora = Mora {
                        vowel_length: phoneme_length[vowel_indexes_data[index + 1] as usize],
                        ..pause_mora.clone()
                    };
                    index += 1;
                    new_pause_mora
                }),
                ..accent_phrase.clone()
            })
            .collect();

        Ok(new_accent_phrases)
    }

    async fn replace_mora_pitch(
        &self,
        accent_phrases: &[AccentPhrase],
        voice_spec: impl VoiceSpec,
    ) -> Result<Vec<AccentPhrase>> {
        let (_, phoneme_data_list) = initial_process(accent_phrases);

        let mut base_start_accent_list = vec![0];
        let mut base_end_accent_list = vec![0];
        let mut base_start_accent_phrase_list = vec![0];
        let mut base_end_accent_phrase_list = vec![0];
        for accent_phrase in accent_phrases {
            let mut accent = usize::from(accent_phrase.accent != 1);
            create_one_accent_list(&mut base_start_accent_list, accent_phrase, accent as i32);

            accent = accent_phrase.accent - 1;
            create_one_accent_list(&mut base_end_accent_list, accent_phrase, accent as i32);
            create_one_accent_list(&mut base_start_accent_phrase_list, accent_phrase, 0);
            create_one_accent_list(&mut base_end_accent_phrase_list, accent_phrase, -1);
        }
        base_start_accent_list.push(0);
        base_end_accent_list.push(0);
        base_start_accent_phrase_list.push(0);
        base_end_accent_phrase_list.push(0);

        let (consonant_phoneme_data_list, vowel_phoneme_data_list, vowel_indexes) =
            split_mora(&phoneme_data_list);

        let consonant_phoneme_list: Vec<i64> = consonant_phoneme_data_list
            .iter()
            .map(|phoneme_data| phoneme_data.phoneme_id())
            .collect();
        let vowel_phoneme_list: Vec<i64> = vowel_phoneme_data_list
            .iter()
            .map(|phoneme_data| phoneme_data.phoneme_id())
            .collect();

        let mut start_accent_list = Vec::with_capacity(vowel_indexes.len());
        let mut end_accent_list = Vec::with_capacity(vowel_indexes.len());
        let mut start_accent_phrase_list = Vec::with_capacity(vowel_indexes.len());
        let mut end_accent_phrase_list = Vec::with_capacity(vowel_indexes.len());

        for vowel_index in vowel_indexes {
            start_accent_list.push(base_start_accent_list[vowel_index as usize]);
            end_accent_list.push(base_end_accent_list[vowel_index as usize]);
            start_accent_phrase_list.push(base_start_accent_phrase_list[vowel_index as usize]);
            end_accent_phrase_list.push(base_end_accent_phrase_list[vowel_index as usize]);
        }

        let mut f0_list = self
            .predict_intonation(
                vowel_phoneme_list.len(),
                &vowel_phoneme_list,
                &consonant_phoneme_list,
                &start_accent_list,
                &end_accent_list,
                &start_accent_phrase_list,
                &end_accent_phrase_list,
                voice_spec,
            )
            .await?;

        for i in 0..vowel_phoneme_data_list.len() {
            const UNVOICED_MORA_PHONEME_LIST: &[&str] = &["A", "I", "U", "E", "O", "cl", "pau"];

            if UNVOICED_MORA_PHONEME_LIST
                .iter()
                .any(|phoneme| *phoneme == vowel_phoneme_data_list[i].phoneme())
            {
                f0_list[i] = 0.;
            }
        }

        let mut index = 0;
        let new_accent_phrases = accent_phrases
            .iter()
            .map(|accent_phrase| AccentPhrase {
                moras: accent_phrase
                    .moras
                    .iter()
                    .map(|mora| {
                        let new_mora = Mora {
                            pitch: f0_list[index + 1],
                            ..mora.clone()
                        };
                        index += 1;
                        new_mora
                    })
                    .collect(),
                pause_mora: accent_phrase.pause_mora.as_ref().map(|pause_mora| {
                    let new_pause_mora = Mora {
                        pitch: f0_list[index + 1],
                        ..pause_mora.clone()
                    };
                    index += 1;
                    new_pause_mora
                }),
                ..accent_phrase.clone()
            })
            .collect();

        return Ok(new_accent_phrases);

        fn create_one_accent_list(
            accent_list: &mut Vec<i64>,
            accent_phrase: &AccentPhrase,
            point: i32,
        ) {
            let mut one_accent_list: Vec<i64> = Vec::new();

            for (i, mora) in accent_phrase.moras.iter().enumerate() {
                let value = (i as i32 == point
                    || (point < 0 && i == (accent_phrase.moras.len() as i32 + point) as usize))
                    .into();
                one_accent_list.push(value);
                if mora.consonant.is_some() {
                    one_accent_list.push(value);
                }
            }
            if accent_phrase.pause_mora.is_some() {
                one_accent_list.push(0);
            }
            accent_list.extend(one_accent_list)
        }
    }

    async fn create_audio_query_from_kana(
        &self,
        kana: &str,
        voice_spec: impl VoiceSpec,
    ) -> Result<AudioQuery> {
        let accent_phrases = self
            .create_accent_phrases_from_kana(kana, voice_spec)
            .await?;
        Ok(AudioQuery::from_accent_phrases(accent_phrases).with_kana(Some(kana.to_owned())))
    }

    async fn tts_from_kana(
        &self,
        kana: &str,
        mut voice_spec: impl VoiceSpec,
        options: &TtsOptions<Self::Async>,
    ) -> Result<Vec<u8>> {
        let audio_query = &self
            .create_audio_query_from_kana(kana, voice_spec.as_mut())
            .await?;
        self.synthesis(audio_query, voice_spec, &SynthesisOptions::from(options))
            .await
    }

    async fn create_accent_phrases(
        &self,
        text: &str,
        voice_spec: impl VoiceSpec,
    ) -> Result<Vec<AccentPhrase>>
    where
        Self::TextAnalyzer: crate::nonblocking::TextAnalyzer,
    {
        let accent_phrases = self.text_analyzer().analyze_(text).await?;
        self.replace_mora_data(&accent_phrases, voice_spec).await
    }

    async fn create_audio_query(&self, text: &str, voice_spec: impl VoiceSpec) -> Result<AudioQuery>
    where
        Self::TextAnalyzer: crate::nonblocking::TextAnalyzer,
    {
        let accent_phrases = self.create_accent_phrases(text, voice_spec).await?;
        Ok(accent_phrases.into())
    }

    async fn tts(
        &self,
        text: &str,
        mut voice_spec: impl VoiceSpec,
        options: &TtsOptions<Self::Async>,
    ) -> Result<Vec<u8>>
    where
        Self::TextAnalyzer: crate::nonblocking::TextAnalyzer,
    {
        let audio_query = &self.create_audio_query(text, voice_spec.as_mut()).await?;
        self.synthesis(audio_query, voice_spec, &SynthesisOptions::from(options))
            .await
    }

    // TODO: この層を破壊する
    async fn predict_duration(
        &self,
        phoneme_vector: &[i64],
        voice_spec: impl VoiceSpec,
    ) -> Result<Vec<f32>> {
        let status = self.status().clone();
        let phoneme_vector = ndarray::arr1(phoneme_vector);
        status
            .predict_duration::<Self::Async>(phoneme_vector, voice_spec)
            .await
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "compatible_engineでの`predict_intonation`の形を考えると、ここの引数を構造体に\
                  まとめたりしても可読性に寄与しない"
    )]
    async fn predict_intonation(
        &self,
        length: usize,
        vowel_phoneme_vector: &[i64],
        consonant_phoneme_vector: &[i64],
        start_accent_vector: &[i64],
        end_accent_vector: &[i64],
        start_accent_phrase_vector: &[i64],
        end_accent_phrase_vector: &[i64],
        voice_spec: impl VoiceSpec,
    ) -> Result<Vec<f32>> {
        let status = self.status().clone();
        let vowel_phoneme_vector = ndarray::arr1(vowel_phoneme_vector);
        let consonant_phoneme_vector = ndarray::arr1(consonant_phoneme_vector);
        let start_accent_vector = ndarray::arr1(start_accent_vector);
        let end_accent_vector = ndarray::arr1(end_accent_vector);
        let start_accent_phrase_vector = ndarray::arr1(start_accent_phrase_vector);
        let end_accent_phrase_vector = ndarray::arr1(end_accent_phrase_vector);
        status
            .predict_intonation::<Self::Async>(
                length,
                vowel_phoneme_vector,
                consonant_phoneme_vector,
                start_accent_vector,
                end_accent_vector,
                start_accent_phrase_vector,
                end_accent_phrase_vector,
                voice_spec,
            )
            .await
    }

    async fn generate_full_intermediate(
        &self,
        length: usize,
        phoneme_size: usize,
        f0: &[f32],
        phoneme_vector: &[f32],
        voice_spec: impl VoiceSpec,
    ) -> Result<ndarray::Array2<f32>> {
        let status = self.status().clone();
        let f0 = ndarray::arr1(f0);
        let phoneme_vector = ndarray::arr1(phoneme_vector);
        status
            .generate_full_intermediate::<Self::Async>(
                length,
                phoneme_size,
                f0,
                phoneme_vector,
                voice_spec,
            )
            .await
    }

    async fn render_audio_segment(
        &self,
        spec: ndarray::Array2<f32>,
        voice_spec: impl VoiceSpec,
    ) -> Result<ndarray::Array1<f32>> {
        let status = self.status().clone();
        status
            .render_audio_segment::<Self::Async>(spec, voice_spec)
            .await
    }

    async fn decode(
        &self,
        length: usize,
        phoneme_size: usize,
        f0: &[f32],
        phoneme_vector: &[f32],
        voice_spec: impl VoiceSpec,
        cancellable: <Self::Async as infer::AsyncExt>::Cancellable,
    ) -> Result<Vec<f32>> {
        let status = self.status().clone();
        let f0 = ndarray::arr1(f0);
        let phoneme_vector = ndarray::arr1(phoneme_vector);
        status
            .decode::<Self::Async>(
                length,
                phoneme_size,
                f0,
                phoneme_vector,
                voice_spec,
                cancellable,
            )
            .await
    }
}

impl<T, A: AsyncExt> AsInner for Inner<T, A> {
    type TextAnalyzer = T;
    type Async = A;

    fn status(&self) -> &Arc<Status<crate::blocking::Onnxruntime>> {
        &self.status
    }

    fn text_analyzer(&self) -> &Self::TextAnalyzer {
        &self.text_analyzer
    }

    fn use_gpu(&self) -> bool {
        self.use_gpu
    }
}

impl<A: AsyncExt> AsInner for InnerRefWithoutTextAnalyzer<'_, A> {
    type TextAnalyzer = ();
    type Async = A;

    fn status(&self) -> &Arc<Status<crate::blocking::Onnxruntime>> {
        self.status
    }

    fn text_analyzer(&self) -> &Self::TextAnalyzer {
        &()
    }

    fn use_gpu(&self) -> bool {
        self.use_gpu
    }
}

impl<R: InferenceRuntime> Status<R> {
    async fn predict_duration<A: infer::AsyncExt>(
        &self,
        phoneme_vector: ndarray::Array1<i64>,
        voice_spec: impl VoiceSpec,
    ) -> Result<Vec<f32>> {
        // `TalkDomain`と`ExperimentalTalkDomain`の両方がある場合、`TalkDomain`を優先
        let (model_id, style_id) = self.find_voice(voice_spec)?;
        if self.contains_domain::<TalkDomain>(style_id) {
            let inner_voice_id = self.inner_voice_id::<TalkDomain>(model_id, style_id);
            let talk::PredictDurationOutput {
                phoneme_length: output,
            } = self
                .run_session::<A, _>(
                    model_id,
                    talk::PredictDurationInput {
                        phoneme_list: phoneme_vector,
                        speaker_id: ndarray::arr1(&[inner_voice_id.raw_id().into()]),
                    },
                    A::LIGHT_INFERENCE_CANCELLABLE,
                )
                .await?;
            return Ok(ensure_minimum_phoneme_length(output.into_raw_vec()));
        }
        let inner_voice_id = self.inner_voice_id::<ExperimentalTalkDomain>(model_id, style_id);

        let experimental_talk::PredictDurationOutput {
            phoneme_length: output,
        } = self
            .run_session::<A, _>(
                model_id,
                experimental_talk::PredictDurationInput {
                    phoneme_list: phoneme_vector,
                    speaker_id: ndarray::arr1(&[inner_voice_id.raw_id().into()]),
                },
                A::LIGHT_INFERENCE_CANCELLABLE,
            )
            .await?;
        Ok(ensure_minimum_phoneme_length(output.into_raw_vec()))
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "compatible_engineでの`predict_intonation`の形を考えると、ここの引数を構造体に\
                  まとめたりしても可読性に寄与しない"
    )]
    async fn predict_intonation<A: infer::AsyncExt>(
        &self,
        length: usize,
        vowel_phoneme_vector: ndarray::Array1<i64>,
        consonant_phoneme_vector: ndarray::Array1<i64>,
        start_accent_vector: ndarray::Array1<i64>,
        end_accent_vector: ndarray::Array1<i64>,
        start_accent_phrase_vector: ndarray::Array1<i64>,
        end_accent_phrase_vector: ndarray::Array1<i64>,
        voice_spec: impl VoiceSpec,
    ) -> Result<Vec<f32>> {
        // `TalkDomain`と`ExperimentalTalkDomain`の両方がある場合、`TalkDomain`を優先
        let (model_id, style_id) = self.find_voice(voice_spec)?;
        if self.contains_domain::<TalkDomain>(style_id) {
            let inner_voice_id = self.inner_voice_id::<TalkDomain>(model_id, style_id);
            let talk::PredictIntonationOutput { f0_list: output } = self
                .run_session::<A, _>(
                    model_id,
                    talk::PredictIntonationInput {
                        length: ndarray::arr0(length as i64),
                        vowel_phoneme_list: vowel_phoneme_vector,
                        consonant_phoneme_list: consonant_phoneme_vector,
                        start_accent_list: start_accent_vector,
                        end_accent_list: end_accent_vector,
                        start_accent_phrase_list: start_accent_phrase_vector,
                        end_accent_phrase_list: end_accent_phrase_vector,
                        speaker_id: ndarray::arr1(&[inner_voice_id.raw_id().into()]),
                    },
                    A::LIGHT_INFERENCE_CANCELLABLE,
                )
                .await?;
            return Ok(output.into_raw_vec());
        }
        let inner_voice_id = self.inner_voice_id::<ExperimentalTalkDomain>(model_id, style_id);

        let experimental_talk::PredictIntonationOutput { f0_list: output } = self
            .run_session::<A, _>(
                model_id,
                experimental_talk::PredictIntonationInput {
                    length: ndarray::arr0(length as i64),
                    vowel_phoneme_list: vowel_phoneme_vector,
                    consonant_phoneme_list: consonant_phoneme_vector,
                    start_accent_list: start_accent_vector,
                    end_accent_list: end_accent_vector,
                    start_accent_phrase_list: start_accent_phrase_vector,
                    end_accent_phrase_list: end_accent_phrase_vector,
                    speaker_id: ndarray::arr1(&[inner_voice_id.raw_id().into()]),
                },
                A::LIGHT_INFERENCE_CANCELLABLE,
            )
            .await?;

        Ok(output.into_raw_vec())
    }

    /// モデル`generate_full_intermediate`の実行と、その前後の処理を行う。
    ///
    /// 無音パディングを付加して音声特徴量を計算し、マージン込みの音声特徴量を返す。
    async fn generate_full_intermediate<A: infer::AsyncExt>(
        &self,
        length: usize,
        phoneme_size: usize,
        f0: ndarray::Array1<f32>,
        phoneme_vector: ndarray::Array1<f32>,
        voice_spec: impl VoiceSpec,
    ) -> Result<ndarray::Array2<f32>> {
        let (model_id, style_id) = self.find_voice(voice_spec)?;
        let inner_voice_id = self.inner_voice_id::<ExperimentalTalkDomain>(model_id, style_id);

        let (length_with_padding, f0_with_padding, phoneme_with_padding) =
            pad_decoder_feature::<PADDING_FRAME_LENGTH>(
                f0,
                phoneme_vector.into_shape([length, phoneme_size]).unwrap(),
            );

        let GenerateFullIntermediateOutput {
            spec: spec_with_padding,
        } = self
            .run_session::<A, _>(
                model_id,
                GenerateFullIntermediateInput {
                    f0: f0_with_padding
                        .into_shape([length_with_padding, 1])
                        .unwrap(),
                    phoneme: phoneme_with_padding,
                    speaker_id: ndarray::arr1(&[inner_voice_id.raw_id().into()]),
                },
                A::LIGHT_INFERENCE_CANCELLABLE,
            )
            .await?;

        // マージンがデータからはみ出さないことを保証
        // cf. https://github.com/VOICEVOX/voicevox_core/pull/854#discussion_r1803691291
        if MARGIN > PADDING_FRAME_LENGTH {
            unreachable!(
                "Validation error: Too short padding for input, please report this issue on GitHub."
            );
        }
        // マージン分を両端に残して音声特徴量を返す
        Ok(spec_with_padding
            .slice(ndarray::s![
                PADDING_FRAME_LENGTH - MARGIN
                    ..spec_with_padding.nrows() - PADDING_FRAME_LENGTH + MARGIN,
                ..
            ])
            .to_owned())
    }

    /// 与えられた音声特徴量で音声生成。
    async fn render_audio_segment<A: infer::AsyncExt>(
        &self,
        spec: ndarray::Array2<f32>,
        voice_spec: impl VoiceSpec,
    ) -> Result<ndarray::Array1<f32>> {
        let (model_id, _) = self.find_voice(voice_spec)?;
        let RenderAudioSegmentOutput { wave } = self
            .run_session::<A, _>(
                model_id,
                RenderAudioSegmentInput { spec },
                A::DEFAULT_HEAVY_INFERENCE_CANCELLABLE, // TODO: 外部から指定可能にする
            )
            .await?;
        Ok(wave)
    }

    async fn decode<A: infer::AsyncExt>(
        &self,
        length: usize,
        phoneme_size: usize,
        f0: ndarray::Array1<f32>,
        phoneme_vector: ndarray::Array1<f32>,
        voice_spec: impl VoiceSpec,
        cancellable: A::Cancellable,
    ) -> Result<Vec<f32>> {
        // `TalkDomain`と`ExperimentalTalkDomain`の両方がある場合、`TalkDomain`を優先
        let (model_id, style_id) = self.find_voice(voice_spec)?;
        if self.contains_domain::<TalkDomain>(style_id) {
            let inner_voice_id = self.inner_voice_id::<TalkDomain>(model_id, style_id);
            let (length_with_padding, f0_with_padding, phoneme_with_padding) =
                pad_decoder_feature::<PADDING_FRAME_LENGTH>(
                    f0,
                    phoneme_vector.into_shape([length, phoneme_size]).unwrap(),
                );
            let DecodeOutput { wave: output } = self
                .run_session::<A, _>(
                    model_id,
                    DecodeInput {
                        f0: f0_with_padding
                            .into_shape([length_with_padding, 1])
                            .unwrap(),
                        phoneme: phoneme_with_padding,
                        speaker_id: ndarray::arr1(&[inner_voice_id.raw_id().into()]),
                    },
                    cancellable,
                )
                .await?;
            let len = output.len();
            return Ok(output
                .slice_move(ndarray::s![
                    PADDING_FRAME_LENGTH * 256..len - PADDING_FRAME_LENGTH * 256,
                ])
                .as_standard_layout()
                .into_owned()
                .into_raw_vec());
        }
        let intermediate = self
            .generate_full_intermediate::<A>(length, phoneme_size, f0, phoneme_vector, style_id)
            .await?;
        let output_with_margin = self
            .render_audio_segment::<A>(intermediate, style_id)
            .await?;
        let output = trim_margin_from_wave(output_with_margin);
        Ok(output.to_vec())
    }

    async fn predict_sing_consonant_length<A: infer::AsyncExt>(
        &self,
        consonant: ndarray::Array1<i64>,
        vowel: ndarray::Array1<i64>,
        note_duration: ndarray::Array1<i64>,
        voice_spec: impl VoiceSpec,
    ) -> Result<ndarray::Array2<i64>> {
        let (model_id, style_id) = self.find_voice(voice_spec)?;
        let inner_voice_id = self.inner_voice_id::<SingingTeacherDomain>(model_id, style_id);

        let PredictSingConsonantLengthOutput { consonant_lengths } = self
            .run_session::<A, _>(
                model_id,
                PredictSingConsonantLengthInput {
                    consonants: consonant.into_one_row(),
                    vowels: vowel.into_one_row(),
                    note_durations: note_duration.into_one_row(),
                    speaker_id: ndarray::array![inner_voice_id.raw_id().into()],
                },
                A::LIGHT_INFERENCE_CANCELLABLE,
            )
            .await?;

        Ok(consonant_lengths)
    }

    async fn predict_sing_f0<A: infer::AsyncExt>(
        &self,
        phoneme: ndarray::Array1<i64>,
        note: ndarray::Array1<i64>,
        voice_spec: impl VoiceSpec,
    ) -> Result<ndarray::Array2<f32>> {
        let (model_id, style_id) = self.find_voice(voice_spec)?;
        let inner_voice_id = self.inner_voice_id::<SingingTeacherDomain>(model_id, style_id);

        let PredictSingF0Output { f0s } = self
            .run_session::<A, _>(
                model_id,
                PredictSingF0Input {
                    phonemes: phoneme.into_one_row(),
                    notes: note.into_one_row(),
                    speaker_id: ndarray::array![inner_voice_id.raw_id().into()],
                },
                A::LIGHT_INFERENCE_CANCELLABLE,
            )
            .await?;

        Ok(f0s)
    }

    async fn predict_sing_volume<A: infer::AsyncExt>(
        &self,
        phoneme: ndarray::Array1<i64>,
        note: ndarray::Array1<i64>,
        f0: ndarray::Array1<f32>,
        voice_spec: impl VoiceSpec,
    ) -> Result<ndarray::Array2<f32>> {
        let (model_id, style_id) = self.find_voice(voice_spec)?;
        let inner_voice_id = self.inner_voice_id::<SingingTeacherDomain>(model_id, style_id);

        let PredictSingVolumeOutput { volumes } = self
            .run_session::<A, _>(
                model_id,
                PredictSingVolumeInput {
                    phonemes: phoneme.into_one_row(),
                    notes: note.into_one_row(),
                    frame_f0s: f0.into_one_row(),
                    speaker_id: ndarray::array![inner_voice_id.raw_id().into()],
                },
                A::LIGHT_INFERENCE_CANCELLABLE,
            )
            .await?;

        Ok(volumes)
    }

    async fn sf_decode<A: infer::AsyncExt>(
        &self,
        phoneme: ndarray::Array1<i64>,
        f0: ndarray::Array1<f32>,
        volume: ndarray::Array1<f32>,
        voice_spec: impl VoiceSpec,
        cancellable: A::Cancellable,
    ) -> Result<ndarray::Array2<f32>> {
        let (model_id, style_id) = self.find_voice(voice_spec)?;
        let inner_voice_id = self.inner_voice_id::<FrameDecodeDomain>(model_id, style_id);

        let SfDecodeOutput { wav } = self
            .run_session::<A, _>(
                model_id,
                SfDecodeInput {
                    frame_phonemes: phoneme.into_one_row(),
                    frame_f0s: f0.into_one_row(),
                    frame_volumes: volume.into_one_row(),
                    speaker_id: ndarray::array![inner_voice_id.raw_id().into()],
                },
                cancellable,
            )
            .await?;

        Ok(wav)
    }
}

#[ext]
impl<T> ndarray::Array1<T> {
    fn into_one_row(self) -> ndarray::Array2<T> {
        let n = self.len();
        self.into_shape([1, n]).expect("should be ok")
    }
}

#[cfg(windows)]
fn list_windows_video_cards() {
    use std::{ffi::OsString, os::windows::ffi::OsStringExt as _};

    use humansize::BINARY;
    use tracing::{error, info};
    use windows::Win32::Graphics::Dxgi::{
        CreateDXGIFactory, IDXGIFactory, DXGI_ADAPTER_DESC, DXGI_ERROR_NOT_FOUND,
    };

    info!("検出されたGPU (DirectMLにはGPU 0が使われます):");
    match list_windows_video_cards() {
        Ok(descs) => {
            for (device_id, desc) in descs.into_iter().enumerate() {
                let description = OsString::from_wide(trim_nul(&desc.Description));
                let vram = humansize::format_size(desc.DedicatedVideoMemory, BINARY);
                info!("  GPU {device_id}: {description:?} ({vram})");
            }
        }
        Err(err) => error!("{err}"),
    }

    fn list_windows_video_cards() -> windows::core::Result<Vec<DXGI_ADAPTER_DESC>> {
        unsafe {
            let factory = CreateDXGIFactory::<IDXGIFactory>()?;
            (0..)
                .map(|i| factory.EnumAdapters(i)?.GetDesc())
                .take_while(|r| !matches!(r, Err(e) if e.code() == DXGI_ERROR_NOT_FOUND))
                .collect()
        }
    }

    fn trim_nul(s: &[u16]) -> &[u16] {
        &s[..s.iter().position(|&c| c == 0x0000).unwrap_or(s.len())]
    }
}

impl AudioQuery {
    /// アクセント句の配列からAudioQueryを作る。
    #[cfg_attr(doc, doc(alias = "voicevox_audio_query_create_from_accent_phrases"))]
    pub fn from_accent_phrases(accent_phrases: Vec<AccentPhrase>) -> Self {
        let kana = create_kana(&accent_phrases);
        Self {
            accent_phrases,
            kana: Some(kana),
            ..Default::default()
        }
    }
}

#[ext(BlockingTextAnalyzerExt)]
impl<T: crate::blocking::TextAnalyzer> T {
    pub fn analyze_(&self, text: &str) -> crate::Result<Vec<AccentPhrase>> {
        self.analyze(text).map_err(|source| {
            ErrorRepr::AnalyzeText {
                text: text.to_owned(),
                source,
            }
            .into()
        })
    }
}

#[ext(NonblockingTextAnalyzerExt)]
impl<T: crate::nonblocking::TextAnalyzer> T {
    pub fn analyze_(
        &self,
        text: &str,
    ) -> impl Future<Output = crate::Result<Vec<AccentPhrase>>> + Send {
        self.analyze(text).map_err(|source| {
            ErrorRepr::AnalyzeText {
                text: text.to_owned(),
                source,
            }
            .into()
        })
    }
}

impl Default for AudioQuery {
    fn default() -> Self {
        Self {
            accent_phrases: vec![],
            speed_scale: 1.,
            pitch_scale: 0.,
            intonation_scale: 1.,
            volume_scale: 1.,
            pre_phoneme_length: 0.1,
            post_phoneme_length: 0.1,
            output_sampling_rate: DEFAULT_SAMPLING_RATE,
            output_stereo: false,
            kana: None,
        }
    }
}

impl From<Vec<AccentPhrase>> for AudioQuery {
    fn from(accent_phrases: Vec<AccentPhrase>) -> Self {
        Self::from_accent_phrases(accent_phrases)
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "`PerformInference::predict_intonation`用。compatible_engineでの`predict_intonation`の\
              形を考えると、ここの引数を構造体にまとめたりしても可読性に寄与しない"
)]
pub(crate) mod blocking {
    use std::{
        fmt::{self, Debug},
        ops::Range,
    };

    use easy_ext::ext;

    use crate::{
        asyncs::SingleTasked, core::voice_spec::VoiceSpec, future::FutureExt as _, AccentPhrase,
        AudioQuery, StyleId, VoiceModelId, VoiceModelMeta,
    };

    use super::{
        AccelerationMode, AsInner as _, AssumeSingleTasked, InitializeOptions, Inner,
        InnerRefWithoutTextAnalyzer, SynthesisOptions, TtsOptions,
    };

    pub use super::AudioFeature;

    /// 音声シンセサイザ。
    #[cfg_attr(doc, doc(alias = "VoicevoxSynthesizer"))]
    pub struct Synthesizer<T>(pub(super) Inner<AssumeSingleTasked<T>, SingleTasked>);

    impl self::Synthesizer<()> {
        /// `Synthesizer`のビルダーをコンストラクトする。
        ///
        /// # Example
        ///
        #[cfg_attr(feature = "load-onnxruntime", doc = "```")]
        #[cfg_attr(not(feature = "load-onnxruntime"), doc = "```compile_fail")]
        /// # fn main() -> anyhow::Result<()> {
        /// # use test_util::OPEN_JTALK_DIC_DIR;
        /// #
        /// # const ACCELERATION_MODE: AccelerationMode = AccelerationMode::Cpu;
        /// #
        /// use std::sync::Arc;
        ///
        /// use voicevox_core::{
        ///     blocking::{Onnxruntime, OpenJtalk, Synthesizer},
        ///     AccelerationMode,
        /// };
        ///
        /// # voicevox_core::blocking::Onnxruntime::load_once()
        /// #     .filename(test_util::ONNXRUNTIME_DYLIB_PATH)
        /// #     .perform()?;
        /// #
        /// let syntesizer = Synthesizer::builder(Onnxruntime::load_once().perform()?)
        ///     .text_analyzer(OpenJtalk::new(OPEN_JTALK_DIC_DIR).unwrap())
        ///     .acceleration_mode(ACCELERATION_MODE)
        ///     .build()?;
        /// #
        /// # Ok(())
        /// # }
        /// ```
        #[cfg_attr(doc, doc(alias = "voicevox_synthesizer_new"))]
        pub fn builder(onnxruntime: &'static crate::blocking::Onnxruntime) -> Builder<()> {
            Builder {
                onnxruntime,
                text_analyzer: (),
                options: Default::default(),
            }
        }
    }

    impl<T> self::Synthesizer<T> {
        #[cfg_attr(doc, doc(alias = "voicevox_synthesizer_get_onnxruntime"))]
        pub fn onnxruntime(&self) -> &'static crate::blocking::Onnxruntime {
            self.0.onnxruntime()
        }

        /// テキスト解析器。
        pub fn text_analyzer(&self) -> &T {
            &self.0.text_analyzer().0
        }

        /// ハードウェアアクセラレーションがGPUモードか判定する。
        #[cfg_attr(doc, doc(alias = "voicevox_synthesizer_is_gpu_mode"))]
        pub fn is_gpu_mode(&self) -> bool {
            self.0.is_gpu_mode()
        }

        /// 音声モデルを読み込む。
        #[cfg_attr(doc, doc(alias = "voicevox_synthesizer_load_voice_model"))]
        pub fn load_voice_model(
            &self,
            model: &crate::blocking::VoiceModelFile,
        ) -> crate::Result<()> {
            self.0.load_voice_model(model.inner()).block_on()
        }

        /// 音声モデルの読み込みを解除する。
        #[cfg_attr(doc, doc(alias = "voicevox_synthesizer_unload_voice_model"))]
        pub fn unload_voice_model(&self, voice_model_id: VoiceModelId) -> crate::Result<()> {
            self.0.unload_voice_model(voice_model_id)
        }

        /// 指定したIDの音声モデルが読み込まれているか判定する。
        #[cfg_attr(doc, doc(alias = "voicevox_synthesizer_is_loaded_voice_model"))]
        pub fn is_loaded_voice_model(&self, voice_model_id: VoiceModelId) -> bool {
            self.0.is_loaded_voice_model(voice_model_id)
        }

        #[doc(hidden)]
        pub fn is_loaded_model_by_style_id(&self, style_id: StyleId) -> bool {
            self.0.is_loaded_model_by_style_id(style_id)
        }

        /// 今読み込んでいる音声モデルのメタ情報を返す。
        #[cfg_attr(doc, doc(alias = "voicevox_synthesizer_create_metas_json"))]
        pub fn metas(&self) -> VoiceModelMeta {
            self.0.metas()
        }

        /// AudioQueryから音声合成用の中間表現を生成する。
        // TODO: 後で復活させる
        // https://github.com/VOICEVOX/voicevox_core/issues/970
        #[doc(hidden)]
        pub fn __precompute_render<'a, S: VoiceSpec>(
            &'a self,
            audio_query: &'a AudioQuery,
            voice_spec: S,
        ) -> PrecomputeRender<'a, S> {
            PrecomputeRender {
                synthesizer: self.0.without_text_analyzer(),
                audio_query,
                voice_spec,
                options: Default::default(),
            }
        }

        /// 中間表現から16bit PCMで音声波形を生成する。
        // TODO: 後で復活させる
        // https://github.com/VOICEVOX/voicevox_core/issues/970
        #[doc(hidden)]
        pub fn __render(
            &self,
            audio: &AudioFeature,
            range: Range<usize>,
        ) -> crate::Result<Vec<u8>> {
            self.0.render(audio, range).block_on()
        }

        /// AudioQueryから直接WAVフォーマットで音声波形を生成する。
        #[cfg_attr(doc, doc(alias = "voicevox_synthesizer_synthesis"))]
        pub fn synthesis<'a, S: VoiceSpec>(
            &'a self,
            audio_query: &'a AudioQuery,
            voice_spec: S,
        ) -> Synthesis<'a, S> {
            Synthesis {
                synthesizer: self.0.without_text_analyzer(),
                audio_query,
                voice_spec,
                options: Default::default(),
            }
        }

        /// AquesTalk風記法からAccentPhrase (アクセント句)の配列を生成する。
        ///
        /// # Example
        ///
        /// ```
        /// # fn main() -> anyhow::Result<()> {
        /// # use pollster::FutureExt as _;
        /// # use voicevox_core::__internal::doctest_fixtures::IntoBlocking as _;
        /// #
        /// # let synthesizer =
        /// #     voicevox_core::__internal::doctest_fixtures::synthesizer_with_sample_voice_model(
        /// #         test_util::SAMPLE_VOICE_MODEL_FILE_PATH,
        /// #         test_util::ONNXRUNTIME_DYLIB_PATH,
        /// #         test_util::OPEN_JTALK_DIC_DIR,
        /// #     )
        /// #     .block_on()?
        /// #     .into_blocking();
        /// #
        /// use voicevox_core::StyleId;
        ///
        /// let accent_phrases = synthesizer
        ///     .create_accent_phrases_from_kana("コンニチワ'", StyleId::new(302))?;
        /// #
        /// # Ok(())
        /// # }
        /// ```
        #[cfg_attr(
            doc,
            doc(alias = "voicevox_synthesizer_create_accent_phrases_from_kana")
        )]
        pub fn create_accent_phrases_from_kana(
            &self,
            kana: &str,
            voice_spec: impl VoiceSpec,
        ) -> crate::Result<Vec<AccentPhrase>> {
            self.0
                .create_accent_phrases_from_kana(kana, voice_spec)
                .block_on()
        }

        /// AccentPhraseの配列の音高・音素長を、特定の声で生成しなおす。
        ///
        /// [`replace_phoneme_length`]と[`replace_mora_pitch`]が一体になったショートハンド。詳細は[音声の調整]の節。
        ///
        /// [`replace_phoneme_length`]: Self::replace_phoneme_length
        /// [`replace_mora_pitch`]: Self::replace_mora_pitch
        /// [音声の調整]: ../index.html#音声の調整
        #[cfg_attr(doc, doc(alias = "voicevox_synthesizer_replace_mora_data"))]
        pub fn replace_mora_data(
            &self,
            accent_phrases: &[AccentPhrase],
            voice_spec: impl VoiceSpec,
        ) -> crate::Result<Vec<AccentPhrase>> {
            self.0
                .replace_mora_data(accent_phrases, voice_spec)
                .block_on()
        }

        /// AccentPhraseの配列の音素長を、特定の声で生成しなおす。
        #[cfg_attr(doc, doc(alias = "voicevox_synthesizer_replace_phoneme_length"))]
        pub fn replace_phoneme_length(
            &self,
            accent_phrases: &[AccentPhrase],
            voice_spec: impl VoiceSpec,
        ) -> crate::Result<Vec<AccentPhrase>> {
            self.0
                .replace_phoneme_length(accent_phrases, voice_spec)
                .block_on()
        }

        /// AccentPhraseの配列の音高を、特定の声で生成しなおす。
        #[cfg_attr(doc, doc(alias = "voicevox_synthesizer_replace_mora_pitch"))]
        pub fn replace_mora_pitch(
            &self,
            accent_phrases: &[AccentPhrase],
            voice_spec: impl VoiceSpec,
        ) -> crate::Result<Vec<AccentPhrase>> {
            self.0
                .replace_mora_pitch(accent_phrases, voice_spec)
                .block_on()
        }

        /// AquesTalk風記法から[AudioQuery]を生成する。
        ///
        /// # Example
        ///
        /// ```
        /// # fn main() -> anyhow::Result<()> {
        /// # use pollster::FutureExt as _;
        /// # use voicevox_core::__internal::doctest_fixtures::IntoBlocking as _;
        /// #
        /// # let synthesizer =
        /// #     voicevox_core::__internal::doctest_fixtures::synthesizer_with_sample_voice_model(
        /// #         test_util::SAMPLE_VOICE_MODEL_FILE_PATH,
        /// #         test_util::ONNXRUNTIME_DYLIB_PATH,
        /// #         test_util::OPEN_JTALK_DIC_DIR,
        /// #     )
        /// #     .block_on()?
        /// #     .into_blocking();
        /// #
        /// use voicevox_core::StyleId;
        ///
        /// let audio_query = synthesizer.create_audio_query_from_kana("コンニチワ'", StyleId::new(302))?;
        /// #
        /// # Ok(())
        /// # }
        /// ```
        ///
        /// [AudioQuery]: crate::AudioQuery
        #[cfg_attr(doc, doc(alias = "voicevox_synthesizer_create_audio_query_from_kana"))]
        pub fn create_audio_query_from_kana(
            &self,
            kana: &str,
            voice_spec: impl VoiceSpec,
        ) -> crate::Result<AudioQuery> {
            self.0
                .create_audio_query_from_kana(kana, voice_spec)
                .block_on()
        }

        /// AquesTalk風記法から音声合成を行う。
        #[cfg_attr(doc, doc(alias = "voicevox_synthesizer_tts_from_kana"))]
        pub fn tts_from_kana<'a, S: VoiceSpec>(
            &'a self,
            kana: &'a str,
            voice_spec: S,
        ) -> TtsFromKana<'a, S> {
            TtsFromKana {
                synthesizer: self.0.without_text_analyzer(),
                kana,
                voice_spec,
                options: TtsOptions::default(),
            }
        }
    }

    impl<T: crate::blocking::TextAnalyzer> self::Synthesizer<T> {
        /// 日本語のテキストからAccentPhrase (アクセント句)の配列を生成する。
        ///
        /// [`TextAnalyzer::analyze`]と[`replace_mora_data`]が一体になったショートハンド。詳細は[音声の調整]の節。
        ///
        /// # Example
        ///
        /// ```
        /// # fn main() -> anyhow::Result<()> {
        /// # use pollster::FutureExt as _;
        /// # use voicevox_core::__internal::doctest_fixtures::IntoBlocking as _;
        /// #
        /// # let synthesizer =
        /// #     voicevox_core::__internal::doctest_fixtures::synthesizer_with_sample_voice_model(
        /// #         test_util::SAMPLE_VOICE_MODEL_FILE_PATH,
        /// #         test_util::ONNXRUNTIME_DYLIB_PATH,
        /// #         test_util::OPEN_JTALK_DIC_DIR,
        /// #     )
        /// #     .block_on()?
        /// #     .into_blocking();
        /// #
        /// use voicevox_core::StyleId;
        ///
        /// let accent_phrases = synthesizer.create_accent_phrases("こんにちは", StyleId::new(302))?;
        /// #
        /// # Ok(())
        /// # }
        /// ```
        ///
        /// [`TextAnalyzer::analyze`]: crate::blocking::TextAnalyzer::analyze
        /// [`replace_mora_data`]: Self::replace_mora_data
        /// [音声の調整]: ../index.html#音声の調整
        #[cfg_attr(doc, doc(alias = "voicevox_synthesizer_create_accent_phrases"))]
        pub fn create_accent_phrases(
            &self,
            text: &str,
            voice_spec: impl VoiceSpec,
        ) -> crate::Result<Vec<AccentPhrase>> {
            self.0.create_accent_phrases(text, voice_spec).block_on()
        }

        /// 日本語のテキストから[AudioQuery]を生成する。
        ///
        /// [`create_accent_phrases`]と[`AudioQuery::from_accent_phrases`]が一体になったショートハンド。詳細は[音声の調整]の節。
        ///
        /// # Examples
        ///
        /// ```
        /// # fn main() -> anyhow::Result<()> {
        /// # use pollster::FutureExt as _;
        /// # use voicevox_core::__internal::doctest_fixtures::IntoBlocking as _;
        /// #
        /// # let synthesizer =
        /// #     voicevox_core::__internal::doctest_fixtures::synthesizer_with_sample_voice_model(
        /// #         test_util::SAMPLE_VOICE_MODEL_FILE_PATH,
        /// #         test_util::ONNXRUNTIME_DYLIB_PATH,
        /// #         test_util::OPEN_JTALK_DIC_DIR,
        /// #     )
        /// #     .block_on()?
        /// #     .into_blocking();
        /// #
        /// use voicevox_core::StyleId;
        ///
        /// let audio_query = synthesizer.create_audio_query("こんにちは", StyleId::new(302))?;
        /// #
        /// # Ok(())
        /// # }
        /// ```
        ///
        /// [AudioQuery]: crate::AudioQuery
        /// [`create_accent_phrases`]: Self::create_accent_phrases
        /// [音声の調整]: ../index.html#音声の調整
        #[cfg_attr(doc, doc(alias = "voicevox_synthesizer_create_audio_query"))]
        pub fn create_audio_query(
            &self,
            text: &str,
            voice_spec: impl VoiceSpec,
        ) -> crate::Result<AudioQuery> {
            self.0.create_audio_query(text, voice_spec).block_on()
        }

        /// 日本語のテキストから音声合成を行う。
        ///
        /// [`create_audio_query`]と[`synthesis`]が一体になったショートハンド。詳細は[音声の調整]の節。
        ///
        /// [`create_audio_query`]: Self::create_audio_query
        /// [`synthesis`]: Self::synthesis
        /// [音声の調整]: ../index.html#音声の調整
        #[cfg_attr(doc, doc(alias = "voicevox_synthesizer_tts"))]
        pub fn tts<'a, S: VoiceSpec>(&'a self, text: &'a str, voice_spec: S) -> Tts<'a, T, S> {
            Tts {
                synthesizer: &self.0,
                text,
                voice_spec,
                options: TtsOptions::default(),
            }
        }
    }

    impl<T: Debug> Debug for self::Synthesizer<T> {
        fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            let fmt = fmt.debug_struct("Synthesizer");
            self.0.fill_debug_struct_body(fmt)
        }
    }

    #[ext(PerformInference)]
    impl self::Synthesizer<()> {
        pub fn predict_duration(
            &self,
            phoneme_vector: &[i64],
            style_id: StyleId,
        ) -> crate::Result<Vec<f32>> {
            self.0.predict_duration(phoneme_vector, style_id).block_on()
        }

        pub fn predict_intonation(
            &self,
            length: usize,
            vowel_phoneme_vector: &[i64],
            consonant_phoneme_vector: &[i64],
            start_accent_vector: &[i64],
            end_accent_vector: &[i64],
            start_accent_phrase_vector: &[i64],
            end_accent_phrase_vector: &[i64],
            style_id: StyleId,
        ) -> crate::Result<Vec<f32>> {
            self.0
                .predict_intonation(
                    length,
                    vowel_phoneme_vector,
                    consonant_phoneme_vector,
                    start_accent_vector,
                    end_accent_vector,
                    start_accent_phrase_vector,
                    end_accent_phrase_vector,
                    style_id,
                )
                .block_on()
        }

        pub fn generate_full_intermediate(
            &self,
            length: usize,
            phoneme_size: usize,
            f0: &[f32],
            phoneme_vector: &[f32],
            style_id: StyleId,
        ) -> crate::Result<ndarray::Array2<f32>> {
            self.0
                .generate_full_intermediate(length, phoneme_size, f0, phoneme_vector, style_id)
                .block_on()
        }

        pub fn render_audio_segment(
            &self,
            spec: ndarray::Array2<f32>,
            style_id: StyleId,
        ) -> crate::Result<ndarray::Array1<f32>> {
            self.0.render_audio_segment(spec, style_id).block_on()
        }

        pub fn decode(
            &self,
            length: usize,
            phoneme_size: usize,
            f0: &[f32],
            phoneme_vector: &[f32],
            style_id: StyleId,
        ) -> crate::Result<Vec<f32>> {
            self.0
                .decode(length, phoneme_size, f0, phoneme_vector, style_id, ())
                .block_on()
        }

        pub fn predict_sing_consonant_length(
            &self,
            consonant: ndarray::Array1<i64>,
            vowel: ndarray::Array1<i64>,
            note_duration: ndarray::Array1<i64>,
            style_id: StyleId,
        ) -> crate::Result<ndarray::Array2<i64>> {
            self.0
                .status
                .predict_sing_consonant_length::<SingleTasked>(
                    consonant,
                    vowel,
                    note_duration,
                    style_id,
                )
                .block_on()
        }

        pub fn predict_sing_f0(
            &self,
            phoneme: ndarray::Array1<i64>,
            note: ndarray::Array1<i64>,
            style_id: StyleId,
        ) -> crate::Result<ndarray::Array2<f32>> {
            self.0
                .status
                .predict_sing_f0::<SingleTasked>(phoneme, note, style_id)
                .block_on()
        }

        pub fn predict_sing_volume(
            &self,
            phoneme: ndarray::Array1<i64>,
            note: ndarray::Array1<i64>,
            f0: ndarray::Array1<f32>,
            style_id: StyleId,
        ) -> crate::Result<ndarray::Array2<f32>> {
            self.0
                .status
                .predict_sing_volume::<SingleTasked>(phoneme, note, f0, style_id)
                .block_on()
        }

        pub fn sf_decode(
            &self,
            phoneme: ndarray::Array1<i64>,
            f0: ndarray::Array1<f32>,
            volume: ndarray::Array1<f32>,
            style_id: StyleId,
        ) -> crate::Result<ndarray::Array2<f32>> {
            self.0
                .status
                .sf_decode::<SingleTasked>(phoneme, f0, volume, style_id, ())
                .block_on()
        }
    }

    #[must_use]
    #[derive(Debug)]
    pub struct Builder<T> {
        onnxruntime: &'static crate::blocking::Onnxruntime,
        text_analyzer: T,
        options: InitializeOptions,
    }

    impl<T> Builder<T> {
        /// テキスト解析器。
        pub fn text_analyzer<T2>(self, text_analyzer: T2) -> Builder<T2> {
            Builder {
                text_analyzer,
                onnxruntime: self.onnxruntime,
                options: self.options,
            }
        }

        pub fn acceleration_mode(mut self, acceleration_mode: AccelerationMode) -> Self {
            self.options.acceleration_mode = acceleration_mode;
            self
        }

        /// CPU利用数を指定。0を指定すると環境に合わせたCPUが利用される。
        pub fn cpu_num_threads(mut self, cpu_num_threads: u16) -> Self {
            self.options.cpu_num_threads = cpu_num_threads;
            self
        }

        /// [`Synthesizer`]をコンストラクトする。
        pub fn build(self) -> crate::Result<Synthesizer<T>> {
            Inner::new(
                self.onnxruntime,
                AssumeSingleTasked(self.text_analyzer),
                &self.options,
            )
            .map(Synthesizer)
        }
    }

    #[must_use = "this is a builder. it does nothing until `perform`ed"]
    #[derive(Debug)]
    pub struct PrecomputeRender<'a, S> {
        synthesizer: InnerRefWithoutTextAnalyzer<'a, SingleTasked>,
        audio_query: &'a AudioQuery,
        voice_spec: S,
        options: SynthesisOptions<SingleTasked>,
    }

    impl<S: VoiceSpec> PrecomputeRender<'_, S> {
        pub fn enable_interrogative_upspeak(mut self, enable_interrogative_upspeak: bool) -> Self {
            self.options.enable_interrogative_upspeak = enable_interrogative_upspeak;
            self
        }

        /// 実行する。
        pub fn perform(self) -> crate::Result<AudioFeature> {
            self.synthesizer
                .precompute_render(self.audio_query, self.voice_spec, &self.options)
                .block_on()
        }
    }

    #[must_use = "this is a builder. it does nothing until `perform`ed"]
    #[derive(Debug)]
    pub struct Synthesis<'a, S> {
        synthesizer: InnerRefWithoutTextAnalyzer<'a, SingleTasked>,
        audio_query: &'a AudioQuery,
        voice_spec: S,
        options: SynthesisOptions<SingleTasked>,
    }

    impl<S: VoiceSpec> Synthesis<'_, S> {
        pub fn enable_interrogative_upspeak(mut self, enable_interrogative_upspeak: bool) -> Self {
            self.options.enable_interrogative_upspeak = enable_interrogative_upspeak;
            self
        }

        /// 実行する。
        pub fn perform(self) -> crate::Result<Vec<u8>> {
            self.synthesizer
                .synthesis(self.audio_query, self.voice_spec, &self.options)
                .block_on()
        }
    }

    #[must_use = "this is a builder. it does nothing until `perform`ed"]
    #[derive(Debug)]
    pub struct TtsFromKana<'a, S> {
        synthesizer: InnerRefWithoutTextAnalyzer<'a, SingleTasked>,
        kana: &'a str,
        voice_spec: S,
        options: TtsOptions<SingleTasked>,
    }

    impl<S: VoiceSpec> TtsFromKana<'_, S> {
        pub fn enable_interrogative_upspeak(mut self, enable_interrogative_upspeak: bool) -> Self {
            self.options.enable_interrogative_upspeak = enable_interrogative_upspeak;
            self
        }

        /// 実行する。
        pub fn perform(self) -> crate::Result<Vec<u8>> {
            self.synthesizer
                .tts_from_kana(self.kana, self.voice_spec, &self.options)
                .block_on()
        }
    }

    #[must_use = "this is a builder. it does nothing until `perform`ed"]
    #[derive(Debug)]
    pub struct Tts<'a, T, S> {
        synthesizer: &'a Inner<AssumeSingleTasked<T>, SingleTasked>,
        text: &'a str,
        voice_spec: S,
        options: TtsOptions<SingleTasked>,
    }

    impl<T: crate::blocking::TextAnalyzer, S: VoiceSpec> Tts<'_, T, S> {
        pub fn enable_interrogative_upspeak(mut self, enable_interrogative_upspeak: bool) -> Self {
            self.options.enable_interrogative_upspeak = enable_interrogative_upspeak;
            self
        }

        /// 実行する。
        pub fn perform(self) -> crate::Result<Vec<u8>> {
            self.synthesizer
                .tts(self.text, self.voice_spec, &self.options)
                .block_on()
        }
    }
}

pub(crate) mod nonblocking {
    use std::fmt::{self, Debug};

    use easy_ext::ext;

    use crate::{
        asyncs::BlockingThreadPool, core::voice_spec::VoiceSpec, AccentPhrase, AudioQuery, Result,
        StyleId, VoiceModelId, VoiceModelMeta,
    };

    use super::{
        AccelerationMode, AsInner as _, AssumeBlockable, InitializeOptions, Inner,
        InnerRefWithoutTextAnalyzer, SynthesisOptions, TtsOptions,
    };

    /// 音声シンセサイザ。
    ///
    /// # Performance
    ///
    /// [blocking]クレートにより動いている。詳しくは[`nonblocking`モジュールのドキュメント]を参照。
    ///
    /// [blocking]: https://docs.rs/crate/blocking
    /// [`nonblocking`モジュールのドキュメント]: crate::nonblocking
    pub struct Synthesizer<T>(pub(super) Inner<T, BlockingThreadPool>);

    impl self::Synthesizer<()> {
        /// `Synthesizer`のビルダーをコンストラクトする。
        ///
        /// # Example
        ///
        #[cfg_attr(feature = "load-onnxruntime", doc = "```")]
        #[cfg_attr(not(feature = "load-onnxruntime"), doc = "```compile_fail")]
        /// # #[pollster::main]
        /// # async fn main() -> anyhow::Result<()> {
        /// # use test_util::OPEN_JTALK_DIC_DIR;
        /// #
        /// # const ACCELERATION_MODE: AccelerationMode = AccelerationMode::Cpu;
        /// #
        /// use std::sync::Arc;
        ///
        /// use voicevox_core::{
        ///     nonblocking::{Onnxruntime, OpenJtalk, Synthesizer},
        ///     AccelerationMode,
        /// };
        ///
        /// # voicevox_core::blocking::Onnxruntime::load_once()
        /// #     .filename(test_util::ONNXRUNTIME_DYLIB_PATH)
        /// #     .perform()?;
        /// #
        /// let syntesizer = Synthesizer::builder(Onnxruntime::load_once().perform().await?)
        ///     .text_analyzer(OpenJtalk::new(OPEN_JTALK_DIC_DIR).await.unwrap())
        ///     .acceleration_mode(ACCELERATION_MODE)
        ///     .build()?;
        /// #
        /// # Ok(())
        /// # }
        /// ```
        pub fn builder(onnxruntime: &'static crate::nonblocking::Onnxruntime) -> Builder<()> {
            Builder {
                onnxruntime,
                text_analyzer: (),
                options: Default::default(),
            }
        }
    }

    impl<T: Send + Sync + 'static> self::Synthesizer<T> {
        pub fn onnxruntime(&self) -> &'static crate::nonblocking::Onnxruntime {
            crate::nonblocking::Onnxruntime::from_blocking(self.0.onnxruntime())
        }

        /// テキスト解析器。
        pub fn text_analyzer(&self) -> &T {
            self.0.text_analyzer()
        }

        /// ハードウェアアクセラレーションがGPUモードか判定する。
        pub fn is_gpu_mode(&self) -> bool {
            self.0.is_gpu_mode()
        }

        /// 音声モデルを読み込む。
        pub async fn load_voice_model(
            &self,
            model: &crate::nonblocking::VoiceModelFile,
        ) -> Result<()> {
            self.0.load_voice_model(model.inner()).await
        }

        /// 音声モデルの読み込みを解除する。
        pub fn unload_voice_model(&self, voice_model_id: VoiceModelId) -> Result<()> {
            self.0.unload_voice_model(voice_model_id)
        }

        /// 指定したIDの音声モデルが読み込まれているか判定する。
        pub fn is_loaded_voice_model(&self, voice_model_id: VoiceModelId) -> bool {
            self.0.is_loaded_voice_model(voice_model_id)
        }

        #[doc(hidden)]
        pub fn is_loaded_model_by_style_id(&self, style_id: StyleId) -> bool {
            self.0.is_loaded_model_by_style_id(style_id)
        }

        /// 今読み込んでいる音声モデルのメタ情報を返す。
        pub fn metas(&self) -> VoiceModelMeta {
            self.0.metas()
        }

        /// AudioQueryから音声合成を行う。
        ///
        /// # Caveats
        ///
        /// [`cancellable`]を有効化しない限り、非同期タスクとしてキャンセルしても終わるまで停止しない。
        ///
        /// [`cancellable`]: Synthesis::cancellable
        pub fn synthesis<'a, S: VoiceSpec>(
            &'a self,
            audio_query: &'a AudioQuery,
            voice_spec: S,
        ) -> Synthesis<'a, S> {
            Synthesis {
                synthesizer: self.0.without_text_analyzer(),
                audio_query,
                voice_spec,
                options: Default::default(),
            }
        }

        /// AquesTalk風記法からAccentPhrase (アクセント句)の配列を生成する。
        ///
        /// # Example
        ///
        /// ```
        /// # #[pollster::main]
        /// # async fn main() -> anyhow::Result<()> {
        /// # let synthesizer =
        /// #     voicevox_core::__internal::doctest_fixtures::synthesizer_with_sample_voice_model(
        /// #         test_util::SAMPLE_VOICE_MODEL_FILE_PATH,
        /// #         test_util::ONNXRUNTIME_DYLIB_PATH,
        /// #         test_util::OPEN_JTALK_DIC_DIR,
        /// #     )
        /// #     .await?;
        /// #
        /// use voicevox_core::StyleId;
        ///
        /// let accent_phrases = synthesizer
        ///     .create_accent_phrases_from_kana("コンニチワ'", StyleId::new(302))
        ///     .await?;
        /// #
        /// # Ok(())
        /// # }
        /// ```
        pub async fn create_accent_phrases_from_kana(
            &self,
            kana: &str,
            voice_spec: impl VoiceSpec,
        ) -> Result<Vec<AccentPhrase>> {
            self.0
                .create_accent_phrases_from_kana(kana, voice_spec)
                .await
        }

        /// AccentPhraseの配列の音高・音素長を、特定の声で生成しなおす。
        ///
        /// [`replace_phoneme_length`]と[`replace_mora_pitch`]が一体になったショートハンド。詳細は[音声の調整]の節。
        ///
        /// [`replace_phoneme_length`]: Self::replace_phoneme_length
        /// [`replace_mora_pitch`]: Self::replace_mora_pitch
        /// [音声の調整]: ../index.html#音声の調整
        pub async fn replace_mora_data(
            &self,
            accent_phrases: &[AccentPhrase],
            voice_spec: impl VoiceSpec,
        ) -> Result<Vec<AccentPhrase>> {
            self.0.replace_mora_data(accent_phrases, voice_spec).await
        }

        /// AccentPhraseの配列の音素長を、特定の声で生成しなおす。
        pub async fn replace_phoneme_length(
            &self,
            accent_phrases: &[AccentPhrase],
            voice_spec: impl VoiceSpec,
        ) -> Result<Vec<AccentPhrase>> {
            self.0
                .replace_phoneme_length(accent_phrases, voice_spec)
                .await
        }

        /// AccentPhraseの配列の音高を、特定の声で生成しなおす。
        pub async fn replace_mora_pitch(
            &self,
            accent_phrases: &[AccentPhrase],
            voice_spec: impl VoiceSpec,
        ) -> Result<Vec<AccentPhrase>> {
            self.0.replace_mora_pitch(accent_phrases, voice_spec).await
        }

        /// AquesTalk風記法から[AudioQuery]を生成する。
        ///
        /// # Example
        ///
        /// ```
        /// # #[pollster::main]
        /// # async fn main() -> anyhow::Result<()> {
        /// # let synthesizer =
        /// #     voicevox_core::__internal::doctest_fixtures::synthesizer_with_sample_voice_model(
        /// #         test_util::SAMPLE_VOICE_MODEL_FILE_PATH,
        /// #         test_util::ONNXRUNTIME_DYLIB_PATH,
        /// #         test_util::OPEN_JTALK_DIC_DIR,
        /// #     )
        /// #     .await?;
        /// #
        /// use voicevox_core::StyleId;
        ///
        /// let audio_query = synthesizer
        ///     .create_audio_query_from_kana("コンニチワ'", StyleId::new(302))
        ///     .await?;
        /// #
        /// # Ok(())
        /// # }
        /// ```
        ///
        /// [AudioQuery]: crate::AudioQuery
        pub async fn create_audio_query_from_kana(
            &self,
            kana: &str,
            voice_spec: impl VoiceSpec,
        ) -> Result<AudioQuery> {
            self.0.create_audio_query_from_kana(kana, voice_spec).await
        }

        /// AquesTalk風記法から音声合成を行う。
        ///
        /// # Caveats
        ///
        /// [`cancellable`]を有効化しない限り、非同期タスクとしてキャンセルしても終わるまで停止しない。
        ///
        /// [`cancellable`]: TtsFromKana::cancellable
        pub fn tts_from_kana<'a, S: VoiceSpec>(
            &'a self,
            kana: &'a str,
            voice_spec: S,
        ) -> TtsFromKana<'a, S> {
            TtsFromKana {
                synthesizer: self.0.without_text_analyzer(),
                kana,
                voice_spec,
                options: Default::default(),
            }
        }
    }

    impl<T: crate::nonblocking::TextAnalyzer> self::Synthesizer<T> {
        /// 日本語のテキストからAccentPhrase (アクセント句)の配列を生成する。
        ///
        /// [`TextAnalyzer::analyze`]と[`replace_mora_data`]が一体になったショートハンド。詳細は[音声の調整]の節。
        ///
        /// # Example
        ///
        /// ```
        /// # #[pollster::main]
        /// # async fn main() -> anyhow::Result<()> {
        /// # let synthesizer =
        /// #     voicevox_core::__internal::doctest_fixtures::synthesizer_with_sample_voice_model(
        /// #         test_util::SAMPLE_VOICE_MODEL_FILE_PATH,
        /// #         test_util::ONNXRUNTIME_DYLIB_PATH,
        /// #         test_util::OPEN_JTALK_DIC_DIR,
        /// #     )
        /// #     .await?;
        /// #
        /// use voicevox_core::StyleId;
        ///
        /// let accent_phrases = synthesizer
        ///     .create_accent_phrases("こんにちは", StyleId::new(302))
        ///     .await?;
        /// #
        /// # Ok(())
        /// # }
        /// ```
        ///
        /// [`TextAnalyzer::analyze`]: crate::nonblocking::TextAnalyzer::analyze
        /// [`replace_mora_data`]: Self::replace_mora_data
        /// [音声の調整]: ../index.html#音声の調整
        pub async fn create_accent_phrases(
            &self,
            text: &str,
            voice_spec: impl VoiceSpec,
        ) -> Result<Vec<AccentPhrase>> {
            self.0.create_accent_phrases(text, voice_spec).await
        }

        /// 日本語のテキストから[AudioQuery]を生成する。
        ///
        /// [`create_accent_phrases`]と[`AudioQuery::from_accent_phrases`]が一体になったショートハンド。詳細は[音声の調整]の節。
        ///
        /// # Examples
        ///
        /// ```
        /// # #[pollster::main]
        /// # async fn main() -> anyhow::Result<()> {
        /// # let synthesizer =
        /// #     voicevox_core::__internal::doctest_fixtures::synthesizer_with_sample_voice_model(
        /// #         test_util::SAMPLE_VOICE_MODEL_FILE_PATH,
        /// #         test_util::ONNXRUNTIME_DYLIB_PATH,
        /// #         test_util::OPEN_JTALK_DIC_DIR,
        /// #     )
        /// #     .await?;
        /// #
        /// use voicevox_core::StyleId;
        ///
        /// let audio_query = synthesizer
        ///     .create_audio_query("こんにちは", StyleId::new(302))
        ///     .await?;
        /// #
        /// # Ok(())
        /// # }
        /// ```
        ///
        /// [AudioQuery]: crate::AudioQuery
        /// [`create_accent_phrases`]: Self::create_accent_phrases
        /// [音声の調整]: ../index.html#音声の調整
        pub async fn create_audio_query(
            &self,
            text: &str,
            voice_spec: impl VoiceSpec,
        ) -> Result<AudioQuery> {
            self.0.create_audio_query(text, voice_spec).await
        }

        /// 日本語のテキストから音声合成を行う。
        ///
        /// [`create_audio_query`]と[`synthesis`]が一体になったショートハンド。詳細は[音声の調整]の節。
        ///
        /// # Caveats
        ///
        /// [`cancellable`]を有効化しない限り、非同期タスクとしてキャンセルしても終わるまで停止しない。
        ///
        /// [`create_audio_query`]: Self::create_audio_query
        /// [`synthesis`]: Self::synthesis
        /// [音声の調整]: ../index.html#音声の調整
        /// [`cancellable`]: Tts::cancellable
        pub fn tts<'a, S: VoiceSpec>(&'a self, text: &'a str, voice_spec: S) -> Tts<'a, T, S> {
            Tts {
                synthesizer: &self.0,
                text,
                voice_spec,
                options: Default::default(),
            }
        }
    }

    impl<T: Debug> Debug for self::Synthesizer<T> {
        fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            let fmt = fmt.debug_struct("Synthesizer");
            self.0.fill_debug_struct_body(fmt)
        }
    }

    #[ext(IntoBlocking)]
    impl<T> self::Synthesizer<T> {
        pub fn into_blocking(self) -> super::blocking::Synthesizer<AssumeBlockable<T>> {
            let x = self.0;
            super::blocking::Synthesizer(x.into())
        }
    }

    #[must_use]
    #[derive(Debug)]
    pub struct Builder<T> {
        onnxruntime: &'static crate::nonblocking::Onnxruntime,
        text_analyzer: T,
        options: InitializeOptions,
    }

    impl<T> Builder<T> {
        /// テキスト解析器。
        pub fn text_analyzer<T2>(self, text_analyzer: T2) -> Builder<T2> {
            Builder {
                text_analyzer,
                onnxruntime: self.onnxruntime,
                options: self.options,
            }
        }

        pub fn acceleration_mode(mut self, acceleration_mode: AccelerationMode) -> Self {
            self.options.acceleration_mode = acceleration_mode;
            self
        }

        /// CPU利用数を指定。0を指定すると環境に合わせたCPUが利用される。
        ///
        /// # Performance
        ///
        /// 未調査ではあるが、物理コアの数+1とするのが適切な可能性がある
        /// ([VOICEVOX/voicevox_core#902])。
        ///
        /// [VOICEVOX/voicevox_core#902]: https://github.com/VOICEVOX/voicevox_core/issues/902
        pub fn cpu_num_threads(mut self, cpu_num_threads: u16) -> Self {
            self.options.cpu_num_threads = cpu_num_threads;
            self
        }

        /// [`Synthesizer`]をコンストラクトする。
        pub fn build(self) -> crate::Result<Synthesizer<T>> {
            Inner::new(&self.onnxruntime.0, self.text_analyzer, &self.options).map(Synthesizer)
        }
    }

    #[must_use = "this is a builder. it does nothing until `perform`ed"]
    #[derive(Debug)]
    pub struct Synthesis<'a, S> {
        synthesizer: InnerRefWithoutTextAnalyzer<'a, BlockingThreadPool>,
        audio_query: &'a AudioQuery,
        voice_spec: S,
        options: SynthesisOptions<BlockingThreadPool>,
    }

    impl<S: VoiceSpec> Synthesis<'_, S> {
        pub fn enable_interrogative_upspeak(mut self, enable_interrogative_upspeak: bool) -> Self {
            self.options.enable_interrogative_upspeak = enable_interrogative_upspeak;
            self
        }

        /// 音声モデルの実行をキャンセル可能にするかどうか。
        ///
        /// このオプションを有効にすると、負荷がかかっている状況下でハングする可能性がある。そのためデフォルトでは無効化されている。[VOICEVOX/voicevox_core#968]を参照。
        ///
        /// [VOICEVOX/voicevox_core#968]: https://github.com/VOICEVOX/voicevox_core/issues/968
        pub fn cancellable(mut self, cancellable: bool) -> Self {
            self.options.cancellable = cancellable;
            self
        }

        /// 実行する。
        pub async fn perform(self) -> crate::Result<Vec<u8>> {
            self.synthesizer
                .synthesis(self.audio_query, self.voice_spec, &self.options)
                .await
        }
    }

    #[must_use = "this is a builder. it does nothing until `perform`ed"]
    #[derive(Debug)]
    pub struct TtsFromKana<'a, S: VoiceSpec> {
        synthesizer: InnerRefWithoutTextAnalyzer<'a, BlockingThreadPool>,
        kana: &'a str,
        voice_spec: S,
        options: TtsOptions<BlockingThreadPool>,
    }

    impl<S: VoiceSpec> TtsFromKana<'_, S> {
        pub fn enable_interrogative_upspeak(mut self, enable_interrogative_upspeak: bool) -> Self {
            self.options.enable_interrogative_upspeak = enable_interrogative_upspeak;
            self
        }

        /// 音声モデルの実行をキャンセル可能にするかどうか。
        ///
        /// このオプションを有効にすると、負荷がかかっている状況下でハングする可能性がある。そのためデフォルトでは無効化されている。[VOICEVOX/voicevox_core#968]を参照。
        ///
        /// [VOICEVOX/voicevox_core#968]: https://github.com/VOICEVOX/voicevox_core/issues/968
        pub fn cancellable(mut self, cancellable: bool) -> Self {
            self.options.cancellable = cancellable;
            self
        }

        /// 実行する。
        pub async fn perform(self) -> crate::Result<Vec<u8>> {
            self.synthesizer
                .tts_from_kana(self.kana, self.voice_spec, &self.options)
                .await
        }
    }

    #[must_use = "this is a builder. it does nothing until `perform`ed"]
    #[derive(Debug)]
    pub struct Tts<'a, T, S> {
        synthesizer: &'a Inner<T, BlockingThreadPool>,
        text: &'a str,
        voice_spec: S,
        options: TtsOptions<BlockingThreadPool>,
    }

    impl<T: crate::nonblocking::TextAnalyzer, S: VoiceSpec> Tts<'_, T, S> {
        pub fn enable_interrogative_upspeak(mut self, enable_interrogative_upspeak: bool) -> Self {
            self.options.enable_interrogative_upspeak = enable_interrogative_upspeak;
            self
        }

        /// 音声モデルの実行をキャンセル可能にするかどうか。
        ///
        /// このオプションを有効にすると、負荷がかかっている状況下でハングする可能性がある。そのためデフォルトでは無効化されている。[VOICEVOX/voicevox_core#968]を参照。
        ///
        /// [VOICEVOX/voicevox_core#968]: https://github.com/VOICEVOX/voicevox_core/issues/968
        pub fn cancellable(mut self, cancellable: bool) -> Self {
            self.options.cancellable = cancellable;
            self
        }

        /// 実行する。
        pub async fn perform(self) -> crate::Result<Vec<u8>> {
            self.synthesizer
                .tts(self.text, self.voice_spec, &self.options)
                .await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AccelerationMode, AsInner as _, DEFAULT_HEAVY_INFERENCE_CANCELLABLE};
    use crate::{
        asyncs::BlockingThreadPool, engine::talk::Mora, macros::tests::assert_debug_fmt_eq,
        AccentPhrase, Result, StyleId,
    };
    use ::test_util::OPEN_JTALK_DIC_DIR;
    use rstest::rstest;

    #[rstest]
    #[case(Ok(()))]
    #[tokio::test]
    async fn load_model_works(#[case] expected_result_at_initialized: Result<()>) {
        let syntesizer = super::nonblocking::Synthesizer::builder(
            crate::nonblocking::Onnxruntime::from_test_util_data()
                .await
                .unwrap(),
        )
        .acceleration_mode(AccelerationMode::Cpu)
        .build()
        .unwrap();

        let result = syntesizer
            .load_voice_model(&crate::nonblocking::VoiceModelFile::sample().await.unwrap())
            .await;

        assert_debug_fmt_eq!(
            expected_result_at_initialized,
            result,
            "got load_model result",
        );
    }

    #[rstest]
    #[tokio::test]
    async fn is_use_gpu_works() {
        let syntesizer = super::nonblocking::Synthesizer::builder(
            crate::nonblocking::Onnxruntime::from_test_util_data()
                .await
                .unwrap(),
        )
        .acceleration_mode(AccelerationMode::Cpu)
        .build()
        .unwrap();
        assert!(!syntesizer.is_gpu_mode());
    }

    #[rstest]
    #[case(1, true)]
    #[tokio::test]
    async fn is_loaded_model_by_style_id_works(#[case] style_id: u32, #[case] expected: bool) {
        let style_id = StyleId::new(style_id);
        let syntesizer = super::nonblocking::Synthesizer::builder(
            crate::nonblocking::Onnxruntime::from_test_util_data()
                .await
                .unwrap(),
        )
        .acceleration_mode(AccelerationMode::Cpu)
        .build()
        .unwrap();
        assert!(
            !syntesizer.is_loaded_model_by_style_id(style_id),
            "expected is_model_loaded to return false, but got true",
        );
        syntesizer
            .load_voice_model(&crate::nonblocking::VoiceModelFile::sample().await.unwrap())
            .await
            .unwrap();

        assert_eq!(
            syntesizer.is_loaded_model_by_style_id(style_id),
            expected,
            "expected is_model_loaded return value against style_id `{style_id}` is `{expected}`, but got `{}`",
            !expected
        );
    }

    #[rstest]
    #[tokio::test]
    async fn predict_duration_works() {
        let syntesizer = super::nonblocking::Synthesizer::builder(
            crate::nonblocking::Onnxruntime::from_test_util_data()
                .await
                .unwrap(),
        )
        .acceleration_mode(AccelerationMode::Cpu)
        .build()
        .unwrap();

        syntesizer
            .load_voice_model(&crate::nonblocking::VoiceModelFile::sample().await.unwrap())
            .await
            .unwrap();

        // 「こんにちは、音声合成の世界へようこそ」という文章を変換して得た phoneme_vector
        let phoneme_vector = [
            0, 23, 30, 4, 28, 21, 10, 21, 42, 7, 0, 30, 4, 35, 14, 14, 16, 30, 30, 35, 14, 14, 28,
            30, 35, 14, 23, 7, 21, 14, 43, 30, 30, 23, 30, 35, 30, 0,
        ];

        let result = syntesizer
            .0
            .predict_duration(&phoneme_vector, StyleId::new(1))
            .await;

        assert!(result.is_ok(), "{result:?}");
        assert_eq!(result.unwrap().len(), phoneme_vector.len());
    }

    #[rstest]
    #[tokio::test]
    async fn predict_intonation_works() {
        let syntesizer = super::nonblocking::Synthesizer::builder(
            crate::nonblocking::Onnxruntime::from_test_util_data()
                .await
                .unwrap(),
        )
        .acceleration_mode(AccelerationMode::Cpu)
        .build()
        .unwrap();
        syntesizer
            .load_voice_model(&crate::nonblocking::VoiceModelFile::sample().await.unwrap())
            .await
            .unwrap();

        // 「テスト」という文章に対応する入力
        let vowel_phoneme_vector = [0, 14, 6, 30, 0];
        let consonant_phoneme_vector = [-1, 37, 35, 37, -1];
        let start_accent_vector = [0, 1, 0, 0, 0];
        let end_accent_vector = [0, 1, 0, 0, 0];
        let start_accent_phrase_vector = [0, 1, 0, 0, 0];
        let end_accent_phrase_vector = [0, 0, 0, 1, 0];

        let result = syntesizer
            .0
            .predict_intonation(
                vowel_phoneme_vector.len(),
                &vowel_phoneme_vector,
                &consonant_phoneme_vector,
                &start_accent_vector,
                &end_accent_vector,
                &start_accent_phrase_vector,
                &end_accent_phrase_vector,
                StyleId::new(1),
            )
            .await;

        assert!(result.is_ok(), "{result:?}");
        assert_eq!(result.unwrap().len(), vowel_phoneme_vector.len());
    }

    #[rstest]
    #[tokio::test]
    async fn decode_works() {
        let syntesizer = super::nonblocking::Synthesizer::builder(
            crate::nonblocking::Onnxruntime::from_test_util_data()
                .await
                .unwrap(),
        )
        .acceleration_mode(AccelerationMode::Cpu)
        .build()
        .unwrap();
        syntesizer
            .load_voice_model(&crate::nonblocking::VoiceModelFile::sample().await.unwrap())
            .await
            .unwrap();

        // 「テスト」という文章に対応する入力
        const F0_LENGTH: usize = 69;
        let mut f0 = [0.; F0_LENGTH];
        f0[9..24].fill(5.905218);
        f0[37..60].fill(5.565851);

        const PHONEME_SIZE: usize = 45;
        let mut phoneme = [0.; PHONEME_SIZE * F0_LENGTH];
        let mut set_one = |index, range| {
            for i in range {
                phoneme[i * PHONEME_SIZE + index] = 1.;
            }
        };
        set_one(0, 0..9);
        set_one(37, 9..13);
        set_one(14, 13..24);
        set_one(35, 24..30);
        set_one(6, 30..37);
        set_one(37, 37..45);
        set_one(30, 45..60);
        set_one(0, 60..69);

        let result = syntesizer
            .0
            .decode(
                F0_LENGTH,
                PHONEME_SIZE,
                &f0,
                &phoneme,
                StyleId::new(1),
                DEFAULT_HEAVY_INFERENCE_CANCELLABLE,
            )
            .await;

        assert!(result.is_ok(), "{result:?}");
        assert_eq!(result.unwrap().len(), F0_LENGTH * 256);
    }

    #[rstest]
    #[tokio::test]
    async fn predict_sing_f0_works() {
        let syntesizer = super::nonblocking::Synthesizer::builder(
            crate::nonblocking::Onnxruntime::from_test_util_data()
                .await
                .unwrap(),
        )
        .acceleration_mode(AccelerationMode::Cpu)
        .build()
        .unwrap();
        syntesizer
            .load_voice_model(&crate::nonblocking::VoiceModelFile::sample().await.unwrap())
            .await
            .unwrap();

        // 「テスト」という文章に対応する入力
        let phoneme_vector = ndarray::array![0, 37, 14, 35, 6, 37, 30, 0];
        let note_vector = ndarray::array![0, 30, 30, 40, 40, 50, 50, 0];

        let sing_teacher_style_id = StyleId::new(6000);
        let result = syntesizer
            .0
            .status
            .predict_sing_f0::<BlockingThreadPool>(
                phoneme_vector.clone(),
                note_vector,
                sing_teacher_style_id,
            )
            .await;

        assert!(result.is_ok(), "{result:?}");
        assert_eq!(result.unwrap().len(), phoneme_vector.len());
    }

    #[rstest]
    #[tokio::test]
    async fn predict_sing_volume_works() {
        let syntesizer = super::nonblocking::Synthesizer::builder(
            crate::nonblocking::Onnxruntime::from_test_util_data()
                .await
                .unwrap(),
        )
        .acceleration_mode(AccelerationMode::Cpu)
        .build()
        .unwrap();
        syntesizer
            .load_voice_model(&crate::nonblocking::VoiceModelFile::sample().await.unwrap())
            .await
            .unwrap();

        // 「テスト」という文章に対応する入力
        let phoneme_vector = ndarray::array![0, 37, 14, 35, 6, 37, 30, 0];
        let note_vector = ndarray::array![0, 30, 30, 40, 40, 50, 50, 0];
        let f0_vector = ndarray::array![0., 5.905218, 5.905218, 0., 0., 5.565851, 5.565851, 0.];

        let sing_teacher_style_id = StyleId::new(6000);
        let result = syntesizer
            .0
            .status
            .predict_sing_volume::<BlockingThreadPool>(
                phoneme_vector.clone(),
                note_vector,
                f0_vector,
                sing_teacher_style_id,
            )
            .await;

        assert!(result.is_ok(), "{result:?}");
        assert_eq!(result.unwrap().len(), phoneme_vector.len());
    }

    #[rstest]
    #[tokio::test]
    async fn sf_decode_works() {
        let syntesizer = super::nonblocking::Synthesizer::builder(
            crate::nonblocking::Onnxruntime::from_test_util_data()
                .await
                .unwrap(),
        )
        .acceleration_mode(AccelerationMode::Cpu)
        .build()
        .unwrap();
        syntesizer
            .load_voice_model(&crate::nonblocking::VoiceModelFile::sample().await.unwrap())
            .await
            .unwrap();

        // 「テスト」という文章に対応する入力
        const F0_LENGTH: usize = 69;
        let mut f0 = [0.; F0_LENGTH];
        f0[9..24].fill(5.905218);
        f0[37..60].fill(5.565851);

        let mut volume = [0.; F0_LENGTH];
        volume[9..24].fill(0.5);
        volume[24..37].fill(0.2);
        volume[37..60].fill(1.0);

        let mut phoneme = [0; F0_LENGTH];
        let mut set_one = |index, range| {
            for i in range {
                phoneme[i] = index;
            }
        };
        set_one(0, 0..9);
        set_one(37, 9..13);
        set_one(14, 13..24);
        set_one(35, 24..30);
        set_one(6, 30..37);
        set_one(37, 37..45);
        set_one(30, 45..60);
        set_one(0, 60..69);

        let sf_decode_style_id = StyleId::new(3000);
        let result = syntesizer
            .0
            .status
            .sf_decode::<BlockingThreadPool>(
                ndarray::arr1(&phoneme),
                ndarray::arr1(&f0),
                ndarray::arr1(&volume),
                sf_decode_style_id,
                DEFAULT_HEAVY_INFERENCE_CANCELLABLE,
            )
            .await;

        assert!(result.is_ok(), "{result:?}");
        assert_eq!(result.unwrap().len(), F0_LENGTH * 256);
    }

    type TextConsonantVowelData =
        [(&'static [(&'static str, &'static str, &'static str)], usize)];

    // [([(テキスト, 母音, 子音), ...], アクセントの位置), ...] の形式
    const TEXT_CONSONANT_VOWEL_DATA1: &TextConsonantVowelData = &[
        (&[("コ", "k", "o"), ("レ", "r", "e"), ("ワ", "w", "a")], 3),
        (
            &[
                ("テ", "t", "e"),
                ("ス", "s", "U"),
                ("ト", "t", "o"),
                ("デ", "d", "e"),
                ("ス", "s", "U"),
            ],
            1,
        ),
    ];

    const TEXT_CONSONANT_VOWEL_DATA2: &TextConsonantVowelData = &[
        (&[("コ", "k", "o"), ("レ", "r", "e"), ("ワ", "w", "a")], 1),
        (
            &[
                ("テ", "t", "e"),
                ("ス", "s", "U"),
                ("ト", "t", "o"),
                ("デ", "d", "e"),
                ("ス", "s", "U"),
            ],
            3,
        ),
    ];

    #[rstest]
    #[case(
        Input::Japanese("これはテストです"),
        TEXT_CONSONANT_VOWEL_DATA1,
        "コレワ'/テ'_ストデ_ス"
    )]
    #[case(
        Input::Kana("コ'レワ/テ_スト'デ_ス"),
        TEXT_CONSONANT_VOWEL_DATA2,
        "コ'レワ/テ_スト'デ_ス"
    )]
    #[tokio::test]
    async fn create_audio_query_works(
        #[case] input: Input,
        #[case] expected_text_consonant_vowel_data: &TextConsonantVowelData,
        #[case] expected_kana_text: &str,
    ) {
        let syntesizer = super::nonblocking::Synthesizer::builder(
            crate::nonblocking::Onnxruntime::from_test_util_data()
                .await
                .unwrap(),
        )
        .text_analyzer(
            crate::nonblocking::OpenJtalk::new(OPEN_JTALK_DIC_DIR)
                .await
                .unwrap(),
        )
        .acceleration_mode(AccelerationMode::Cpu)
        .build()
        .unwrap();

        let model = &crate::nonblocking::VoiceModelFile::sample().await.unwrap();
        syntesizer.load_voice_model(model).await.unwrap();

        let query = match input {
            Input::Kana(input) => {
                syntesizer
                    .create_audio_query_from_kana(input, StyleId::new(0))
                    .await
            }
            Input::Japanese(input) => syntesizer.create_audio_query(input, StyleId::new(0)).await,
        }
        .unwrap();

        assert_eq!(
            query.accent_phrases.len(),
            expected_text_consonant_vowel_data.len()
        );

        for (accent_phrase, (text_consonant_vowel_slice, accent_pos)) in std::iter::zip(
            query.accent_phrases,
            expected_text_consonant_vowel_data.iter().copied(),
        ) {
            assert_eq!(accent_phrase.moras.len(), text_consonant_vowel_slice.len());
            assert_eq!(accent_phrase.accent, accent_pos);

            for (mora, (text, consonant, vowel)) in std::iter::zip(
                accent_phrase.moras,
                text_consonant_vowel_slice.iter().copied(),
            ) {
                assert_eq!(mora.text, text);
                // NOTE: 子音の長さが必ず非ゼロになるテストケースを想定している
                assert_ne!(
                    mora.consonant_length,
                    Some(0.),
                    "expected mora.consonant_length is not Some(0.0), but got Some(0.0)."
                );
                assert_eq!(mora.consonant, Some(consonant.to_string()));
                assert_eq!(mora.vowel, vowel);
                // NOTE: 母音の長さが必ず非ゼロになるテストケースを想定している
                assert_ne!(
                    mora.vowel_length, 0.,
                    "expected mora.vowel_length is not 0.0, but got 0.0."
                );
            }
        }

        assert_eq!(query.kana.as_deref(), Some(expected_kana_text));
    }

    #[rstest]
    #[case(Input::Japanese("これはテストです"), TEXT_CONSONANT_VOWEL_DATA1)]
    #[case(Input::Kana("コ'レワ/テ_スト'デ_ス"), TEXT_CONSONANT_VOWEL_DATA2)]
    #[tokio::test]
    async fn crate_accent_phrases_works(
        #[case] input: Input,
        #[case] expected_text_consonant_vowel_data: &TextConsonantVowelData,
    ) {
        let syntesizer = super::nonblocking::Synthesizer::builder(
            crate::nonblocking::Onnxruntime::from_test_util_data()
                .await
                .unwrap(),
        )
        .text_analyzer(
            crate::nonblocking::OpenJtalk::new(OPEN_JTALK_DIC_DIR)
                .await
                .unwrap(),
        )
        .acceleration_mode(AccelerationMode::Cpu)
        .build()
        .unwrap();

        let model = &crate::nonblocking::VoiceModelFile::sample().await.unwrap();
        syntesizer.load_voice_model(model).await.unwrap();

        let accent_phrases = match input {
            Input::Kana(input) => {
                syntesizer
                    .create_accent_phrases_from_kana(input, StyleId::new(0))
                    .await
            }
            Input::Japanese(input) => {
                syntesizer
                    .create_accent_phrases(input, StyleId::new(0))
                    .await
            }
        }
        .unwrap();

        assert_eq!(
            accent_phrases.len(),
            expected_text_consonant_vowel_data.len()
        );

        for (accent_phrase, (text_consonant_vowel_slice, accent_pos)) in std::iter::zip(
            accent_phrases,
            expected_text_consonant_vowel_data.iter().copied(),
        ) {
            assert_eq!(accent_phrase.moras.len(), text_consonant_vowel_slice.len());
            assert_eq!(accent_phrase.accent, accent_pos);

            for (mora, (text, consonant, vowel)) in std::iter::zip(
                accent_phrase.moras,
                text_consonant_vowel_slice.iter().copied(),
            ) {
                assert_eq!(mora.text, text);
                // NOTE: 子音の長さが必ず非ゼロになるテストケースを想定している
                assert_ne!(
                    mora.consonant_length,
                    Some(0.),
                    "expected mora.consonant_length is not Some(0.0), but got Some(0.0)."
                );
                assert_eq!(mora.consonant, Some(consonant.to_string()));
                assert_eq!(mora.vowel, vowel);
                // NOTE: 母音の長さが必ず非ゼロになるテストケースを想定している
                assert_ne!(
                    mora.vowel_length, 0.,
                    "expected mora.vowel_length is not 0.0, but got 0.0."
                );
            }
        }
    }

    #[rstest]
    #[tokio::test]
    async fn create_accent_phrases_works_for_japanese_commas_and_periods() {
        let syntesizer = super::nonblocking::Synthesizer::builder(
            crate::nonblocking::Onnxruntime::from_test_util_data()
                .await
                .unwrap(),
        )
        .text_analyzer(
            crate::nonblocking::OpenJtalk::new(OPEN_JTALK_DIC_DIR)
                .await
                .unwrap(),
        )
        .acceleration_mode(AccelerationMode::Cpu)
        .build()
        .unwrap();

        let model = &crate::nonblocking::VoiceModelFile::sample().await.unwrap();
        syntesizer.load_voice_model(model).await.unwrap();

        let accent_phrases = syntesizer
            .create_accent_phrases("同じ、文章、です。完全に、同一です。", StyleId::new(1))
            .await
            .unwrap();
        assert_eq!(accent_phrases.len(), 5);

        // 入力テキストに「、」や「。」などの句読点が含まれていたときに
        // AccentPhraseの pause_mora に期待する値をテスト

        assert!(
            accent_phrases[0].pause_mora.is_some(),
            "accent_phrases[0].pause_mora is None"
        );
        assert!(
            accent_phrases[1].pause_mora.is_some(),
            "accent_phrases[1].pause_mora is None"
        );
        assert!(
            accent_phrases[2].pause_mora.is_some(),
            "accent_phrases[2].pause_mora is None"
        );
        assert!(
            accent_phrases[3].pause_mora.is_some(),
            "accent_phrases[3].pause_mora is None"
        );
        assert!(
            accent_phrases[4].pause_mora.is_none(), // 文末の句読点は削除される
            "accent_phrases[4].pause_mora is not None"
        );

        for accent_phrase in accent_phrases.iter().take(4) {
            let pause_mora = accent_phrase.pause_mora.clone().unwrap();
            assert_eq!(pause_mora.text, "、");
            assert_eq!(pause_mora.consonant, None);
            assert_eq!(pause_mora.consonant_length, None);
            assert_eq!(pause_mora.vowel, "pau");
            assert_ne!(
                pause_mora.vowel_length, 0.0,
                "pause_mora.vowel_length should not be 0.0",
            );
        }
    }

    #[rstest]
    #[tokio::test]
    async fn mora_length_works() {
        let syntesizer = super::nonblocking::Synthesizer::builder(
            crate::nonblocking::Onnxruntime::from_test_util_data()
                .await
                .unwrap(),
        )
        .text_analyzer(
            crate::nonblocking::OpenJtalk::new(OPEN_JTALK_DIC_DIR)
                .await
                .unwrap(),
        )
        .acceleration_mode(AccelerationMode::Cpu)
        .build()
        .unwrap();

        let model = &crate::nonblocking::VoiceModelFile::sample().await.unwrap();
        syntesizer.load_voice_model(model).await.unwrap();

        let accent_phrases = syntesizer
            .create_accent_phrases("これはテストです", StyleId::new(0))
            .await
            .unwrap();

        let modified_accent_phrases = syntesizer
            .replace_phoneme_length(&accent_phrases, StyleId::new(1))
            .await
            .unwrap();

        // NOTE: 一つでも母音の長さが変わっていれば、動作しているとみなす
        assert!(
            any_mora_param_changed(
                &accent_phrases,
                &modified_accent_phrases,
                |Mora { vowel_length, .. }| vowel_length,
            ),
            "mora_length() does not work: mora.vowel_length is not changed.",
        );
    }

    #[rstest]
    #[tokio::test]
    async fn mora_pitch_works() {
        let syntesizer = super::nonblocking::Synthesizer::builder(
            crate::nonblocking::Onnxruntime::from_test_util_data()
                .await
                .unwrap(),
        )
        .text_analyzer(
            crate::nonblocking::OpenJtalk::new(OPEN_JTALK_DIC_DIR)
                .await
                .unwrap(),
        )
        .acceleration_mode(AccelerationMode::Cpu)
        .build()
        .unwrap();

        let model = &crate::nonblocking::VoiceModelFile::sample().await.unwrap();
        syntesizer.load_voice_model(model).await.unwrap();

        let accent_phrases = syntesizer
            .create_accent_phrases("これはテストです", StyleId::new(0))
            .await
            .unwrap();

        let modified_accent_phrases = syntesizer
            .replace_mora_pitch(&accent_phrases, StyleId::new(1))
            .await
            .unwrap();

        // NOTE: 一つでも音高が変わっていれば、動作しているとみなす
        assert!(
            any_mora_param_changed(
                &accent_phrases,
                &modified_accent_phrases,
                |Mora { pitch, .. }| pitch
            ),
            "mora_pitch() does not work: mora.pitch is not changed.",
        );
    }

    #[rstest]
    #[tokio::test]
    async fn mora_data_works() {
        let syntesizer = super::nonblocking::Synthesizer::builder(
            crate::nonblocking::Onnxruntime::from_test_util_data()
                .await
                .unwrap(),
        )
        .text_analyzer(
            crate::nonblocking::OpenJtalk::new(OPEN_JTALK_DIC_DIR)
                .await
                .unwrap(),
        )
        .acceleration_mode(AccelerationMode::Cpu)
        .build()
        .unwrap();

        let model = &crate::nonblocking::VoiceModelFile::sample().await.unwrap();
        syntesizer.load_voice_model(model).await.unwrap();

        let accent_phrases = syntesizer
            .create_accent_phrases("これはテストです", StyleId::new(0))
            .await
            .unwrap();

        let modified_accent_phrases = syntesizer
            .replace_mora_data(&accent_phrases, StyleId::new(1))
            .await
            .unwrap();

        // NOTE: 一つでも音高が変わっていれば、動作しているとみなす
        assert!(
            any_mora_param_changed(
                &accent_phrases,
                &modified_accent_phrases,
                |Mora { pitch, .. }| pitch,
            ),
            "mora_data() does not work: mora.pitch is not changed.",
        );
        // NOTE: 一つでも母音の長さが変わっていれば、動作しているとみなす
        assert!(
            any_mora_param_changed(
                &accent_phrases,
                &modified_accent_phrases,
                |Mora { vowel_length, .. }| vowel_length,
            ),
            "mora_data() does not work: mora.vowel_length is not changed.",
        );
    }

    fn any_mora_param_changed<T: PartialEq>(
        before: &[AccentPhrase],
        after: &[AccentPhrase],
        param: fn(&Mora) -> &T,
    ) -> bool {
        std::iter::zip(before, after)
            .flat_map(|(before, after)| std::iter::zip(&before.moras, &after.moras))
            .any(|(before, after)| param(before) != param(after))
    }

    enum Input {
        Japanese(&'static str),
        Kana(&'static str),
    }
}
