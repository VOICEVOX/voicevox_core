use crate::asyncs::{Async, BlockingThreadPool, SingleTasked};

/// [`blocking::Synthesizer::synthesis`]および[`nonblocking::Synthesizer::synthesis`]のオプション。
///
/// [`blocking::Synthesizer::synthesis`]: blocking::Synthesizer::synthesis
/// [`nonblocking::Synthesizer::synthesis`]: nonblocking::Synthesizer::synthesis
#[derive(Clone)]
pub struct SynthesisOptions {
    pub enable_interrogative_upspeak: bool,
}

impl AsRef<SynthesisOptions> for SynthesisOptions {
    fn as_ref(&self) -> &SynthesisOptions {
        self
    }
}

impl From<&TtsOptions> for SynthesisOptions {
    fn from(options: &TtsOptions) -> Self {
        Self {
            enable_interrogative_upspeak: options.enable_interrogative_upspeak,
        }
    }
}

/// [`blocking::Synthesizer::tts`]および[`nonblocking::Synthesizer::tts`]のオプション。
///
/// [`blocking::Synthesizer::tts`]: blocking::Synthesizer::tts
/// [`nonblocking::Synthesizer::tts`]: nonblocking::Synthesizer::tts
#[derive(Clone)]
pub struct TtsOptions {
    pub enable_interrogative_upspeak: bool,
}

impl AsRef<TtsOptions> for TtsOptions {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl Default for TtsOptions {
    fn default() -> Self {
        Self {
            enable_interrogative_upspeak: true,
        }
    }
}

/// ハードウェアアクセラレーションモードを設定する設定値。
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccelerationMode {
    /// 実行環境に合った適切なハードウェアアクセラレーションモードを選択する。
    #[default]
    Auto,
    /// ハードウェアアクセラレーションモードを"CPU"に設定する。
    Cpu,
    /// ハードウェアアクセラレーションモードを"GPU"に設定する。
    Gpu,
}

/// [`blocking::Synthesizer::new`]および[`nonblocking::Synthesizer::new`]のオプション。
///
/// [`blocking::Synthesizer::new`]: blocking::Synthesizer::new
/// [`nonblocking::Synthesizer::new`]: nonblocking::Synthesizer::new
#[derive(Default)]
pub struct InitializeOptions {
    pub acceleration_mode: AccelerationMode,
    pub cpu_num_threads: u16,
}

trait AsyncForOnnxruntime: Async {
    async fn unblock<T, F>(f: F) -> T
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static;
}

impl AsyncForOnnxruntime for SingleTasked {
    async fn unblock<T, F>(f: F) -> T
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        f()
    }
}

impl AsyncForOnnxruntime for BlockingThreadPool {
    async fn unblock<T, F>(f: F) -> T
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        ::blocking::unblock(f).await
    }
}

mod inner {
    use enum_map::enum_map;
    use std::{
        io::{Cursor, Write as _},
        marker::PhantomData,
        ops::Range,
        sync::Arc,
    };
    use tracing::info;

    use crate::{
        asyncs::{BlockingThreadPool, SingleTasked},
        devices::{DeviceSpec, GpuSpec},
        engine::{create_kana, mora_to_text, wav_from_s16le, Mora, OjtPhoneme},
        error::ErrorRepr,
        infer::{
            domains::{
                GenerateFullIntermediateInput, GenerateFullIntermediateOutput, InferenceDomainMap,
                PredictDurationInput, PredictDurationOutput, PredictIntonationInput,
                PredictIntonationOutput, RenderAudioSegmentInput, RenderAudioSegmentOutput,
                TalkDomain, TalkOperation,
            },
            InferenceRuntime, InferenceSessionOptions,
        },
        status::Status,
        text_analyzer::{KanaAnalyzer, OpenJTalkAnalyzer, TextAnalyzer},
        voice_model, AccentPhrase, AudioQuery, FullcontextExtractor, Result, StyleId,
        SynthesisOptions, VoiceModelId, VoiceModelMeta,
    };

    use super::{AccelerationMode, AsyncForOnnxruntime, InitializeOptions, TtsOptions};

    const DEFAULT_SAMPLING_RATE: u32 = 24000;
    /// 音が途切れてしまうのを避けるworkaround処理のためのパディング幅（フレーム数）
    const PADDING_FRAME_LENGTH: usize = 38; // (0.4秒 * 24000Hz / 256.0).round()
    /// 音声生成の際、音声特徴量の前後に確保すべきマージン幅（フレーム数）
    /// モデルの受容野から計算される
    const MARGIN: usize = 14;
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
    pub struct AudioFeature {
        /// (フレーム数, 特徴数)の形を持つ音声特徴量。
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

    pub struct Inner<O, A> {
        status: Arc<Status<crate::blocking::Onnxruntime>>,
        open_jtalk_analyzer: OpenJTalkAnalyzer<O>,
        kana_analyzer: KanaAnalyzer,
        use_gpu: bool,
        _marker: PhantomData<fn(A) -> A>,
    }

    impl<O> From<Inner<O, BlockingThreadPool>> for Inner<O, SingleTasked> {
        fn from(from: Inner<O, BlockingThreadPool>) -> Self {
            Self {
                status: from.status,
                open_jtalk_analyzer: from.open_jtalk_analyzer,
                kana_analyzer: KanaAnalyzer,
                use_gpu: from.use_gpu,
                _marker: PhantomData,
            }
        }
    }

    impl<O, A: AsyncForOnnxruntime> Inner<O, A> {
        pub(super) fn new(
            onnxruntime: &'static crate::blocking::Onnxruntime,
            open_jtalk: O,
            options: &InitializeOptions,
        ) -> Result<Self> {
            #[cfg(windows)]
            list_windows_video_cards();

            let test_gpus = || {
                info!("GPUをテストします:");
                let availabilities = crate::devices::test_gpus(
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
                        TalkOperation::PredictDuration
                        | TalkOperation::PredictIntonation
                        | TalkOperation::GenerateFullIntermediate => light_session_options,
                        TalkOperation::RenderAudioSegment => heavy_session_options,
                    },
                },
            )
            .into();

            let use_gpu = matches!(device_for_heavy, DeviceSpec::Gpu(_));

            Ok(Self {
                status,
                open_jtalk_analyzer: OpenJTalkAnalyzer::new(open_jtalk),
                kana_analyzer: KanaAnalyzer,
                use_gpu,
                _marker: PhantomData,
            })
        }

        pub(super) fn onnxruntime(&self) -> &'static crate::blocking::Onnxruntime {
            self.status.rt
        }

        pub(super) fn is_gpu_mode(&self) -> bool {
            self.use_gpu
        }

        pub(super) async fn load_voice_model(
            &self,
            model: &voice_model::Inner<A>,
        ) -> crate::Result<()> {
            let model_bytes = model.read_inference_models().await?;

            let status = self.status.clone();
            let header = model.header().clone();
            A::unblock(move || status.insert_model(&header, &model_bytes)).await
        }

        pub(super) fn unload_voice_model(&self, voice_model_id: VoiceModelId) -> Result<()> {
            self.status.unload_model(voice_model_id)
        }

        pub(super) fn is_loaded_voice_model(&self, voice_model_id: VoiceModelId) -> bool {
            self.status.is_loaded_model(voice_model_id)
        }

        pub(super) fn is_loaded_model_by_style_id(&self, style_id: StyleId) -> bool {
            self.status.is_loaded_model_by_style_id(style_id)
        }

        pub(super) fn metas(&self) -> VoiceModelMeta {
            self.status.metas()
        }

        pub(super) async fn precompute_render(
            &self,
            audio_query: &AudioQuery,
            style_id: StyleId,
            options: &SynthesisOptions,
        ) -> Result<AudioFeature> {
            let AudioQuery {
                accent_phrases,
                speed_scale,
                pitch_scale,
                intonation_scale,
                pre_phoneme_length,
                post_phoneme_length,
                ..
            } = audio_query;

            let accent_phrases = if options.enable_interrogative_upspeak {
                &adjust_interrogative_accent_phrases(accent_phrases)
            } else {
                accent_phrases
            };

            let (flatten_moras, phoneme_data_list) = initial_process(accent_phrases);

            let mut phoneme_length_list = vec![*pre_phoneme_length];
            let mut f0_list = vec![0.];
            let mut voiced_list = vec![false];
            {
                let mut sum_of_f0_bigger_than_zero = 0.;
                let mut count_of_f0_bigger_than_zero = 0;

                for Mora {
                    consonant_length,
                    vowel_length,
                    pitch,
                    ..
                } in flatten_moras
                {
                    if let Some(consonant_length) = consonant_length {
                        phoneme_length_list.push(consonant_length);
                    }
                    phoneme_length_list.push(vowel_length);

                    let f0_single = pitch * 2.0_f32.powf(*pitch_scale);
                    f0_list.push(f0_single);

                    let bigger_than_zero = f0_single > 0.;
                    voiced_list.push(bigger_than_zero);

                    if bigger_than_zero {
                        sum_of_f0_bigger_than_zero += f0_single;
                        count_of_f0_bigger_than_zero += 1;
                    }
                }
                phoneme_length_list.push(*post_phoneme_length);
                f0_list.push(0.);
                voiced_list.push(false);
                let mean_f0 = sum_of_f0_bigger_than_zero / (count_of_f0_bigger_than_zero as f32);

                if !mean_f0.is_nan() {
                    for i in 0..f0_list.len() {
                        if voiced_list[i] {
                            f0_list[i] = (f0_list[i] - mean_f0) * intonation_scale + mean_f0;
                        }
                    }
                }
            }

            let (_, _, vowel_indexes) = split_mora(&phoneme_data_list);

            let mut phoneme = Vec::new();
            let mut f0: Vec<f32> = Vec::new();
            {
                const RATE: f32 = 24000. / 256.;
                let mut sum_of_phoneme_length = 0;
                let mut count_of_f0 = 0;
                let mut vowel_indexes_index = 0;

                for (i, phoneme_length) in phoneme_length_list.iter().enumerate() {
                    // VOICEVOX ENGINEと挙動を合わせるため、四捨五入ではなく偶数丸めをする
                    //
                    // https://github.com/VOICEVOX/voicevox_engine/issues/552
                    let phoneme_length = ((*phoneme_length * RATE).round_ties_even() / speed_scale)
                        .round_ties_even() as usize;
                    let phoneme_id = phoneme_data_list[i].phoneme_id();

                    for _ in 0..phoneme_length {
                        let mut phonemes_vec = [0.; OjtPhoneme::num_phoneme()];
                        phonemes_vec[phoneme_id as usize] = 1.;
                        phoneme.push(phonemes_vec)
                    }
                    sum_of_phoneme_length += phoneme_length;

                    if i as i64 == vowel_indexes[vowel_indexes_index] {
                        for _ in 0..sum_of_phoneme_length {
                            f0.push(f0_list[count_of_f0]);
                        }
                        count_of_f0 += 1;
                        sum_of_phoneme_length = 0;
                        vowel_indexes_index += 1;
                    }
                }
            }

            let spec = self
                .generate_full_intermediate(
                    f0.len(),
                    OjtPhoneme::num_phoneme(),
                    &f0,
                    phoneme.as_flattened(),
                    style_id,
                )
                .await?;
            return Ok(AudioFeature {
                internal_state: spec,
                style_id,
                frame_length: f0.len(),
                frame_rate: (DEFAULT_SAMPLING_RATE as f64) / 256.0,
                audio_query: audio_query.clone(),
            });

            fn adjust_interrogative_accent_phrases(
                accent_phrases: &[AccentPhrase],
            ) -> Vec<AccentPhrase> {
                accent_phrases
                    .iter()
                    .map(|accent_phrase| AccentPhrase {
                        moras: adjust_interrogative_moras(accent_phrase),
                        ..accent_phrase.clone()
                    })
                    .collect()
            }

            fn adjust_interrogative_moras(
                AccentPhrase {
                    moras,
                    is_interrogative,
                    ..
                }: &AccentPhrase,
            ) -> Vec<Mora> {
                if *is_interrogative && !moras.is_empty() {
                    let last_mora = moras.last().unwrap();
                    if last_mora.pitch != 0.0 {
                        let mut new_moras: Vec<Mora> = Vec::with_capacity(moras.len() + 1);
                        new_moras.extend_from_slice(moras.as_slice());
                        let interrogative_mora = make_interrogative_mora(last_mora);
                        new_moras.push(interrogative_mora);
                        return new_moras;
                    }
                }
                moras.clone()
            }

            fn make_interrogative_mora(last_mora: &Mora) -> Mora {
                const FIX_VOWEL_LENGTH: f32 = 0.15;
                const ADJUST_PITCH: f32 = 0.3;
                const MAX_PITCH: f32 = 6.5;

                let pitch = (last_mora.pitch + ADJUST_PITCH).min(MAX_PITCH);

                Mora {
                    text: mora_to_text(None, &last_mora.vowel),
                    consonant: None,
                    consonant_length: None,
                    vowel: last_mora.vowel.clone(),
                    vowel_length: FIX_VOWEL_LENGTH,
                    pitch,
                }
            }
        }

        pub(super) async fn render(
            &self,
            audio: &AudioFeature,
            range: Range<usize>,
        ) -> Result<Vec<u8>> {
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
            return Ok(to_s16le_pcm(
                wave.as_slice()
                    .expect("`trim_margin_from_wave` should just trim an array"),
                &audio.audio_query,
            ));

            fn to_s16le_pcm(
                wave: &[f32],
                &AudioQuery {
                    volume_scale,
                    output_sampling_rate,
                    output_stereo,
                    ..
                }: &AudioQuery,
            ) -> Vec<u8> {
                let num_channels: u16 = if output_stereo { 2 } else { 1 };
                let repeat_count: u32 =
                    (output_sampling_rate / DEFAULT_SAMPLING_RATE) * num_channels as u32;
                let bytes_size = wave.len() as u32 * repeat_count * 2;
                let buf: Vec<u8> = Vec::with_capacity(bytes_size as usize);
                let mut cur = Cursor::new(buf);

                for value in wave {
                    let v = (value * volume_scale).clamp(-1., 1.);
                    let data = (v * 0x7fff as f32) as i16;
                    for _ in 0..repeat_count {
                        cur.write_all(&data.to_le_bytes()).unwrap();
                    }
                }

                cur.into_inner()
            }
        }

        pub(super) async fn synthesis(
            &self,
            audio_query: &AudioQuery,
            style_id: StyleId,
            options: &SynthesisOptions,
        ) -> Result<Vec<u8>> {
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

        pub(super) async fn create_accent_phrases_from_kana(
            &self,
            kana: &str,
            style_id: StyleId,
        ) -> Result<Vec<AccentPhrase>> {
            let accent_phrases = self.kana_analyzer.analyze(kana)?;
            self.replace_mora_data(&accent_phrases, style_id).await
        }

        pub(super) async fn replace_mora_data(
            &self,
            accent_phrases: &[AccentPhrase],
            style_id: StyleId,
        ) -> Result<Vec<AccentPhrase>> {
            let accent_phrases = self
                .replace_phoneme_length(accent_phrases, style_id)
                .await?;
            self.replace_mora_pitch(&accent_phrases, style_id).await
        }

        pub(super) async fn replace_phoneme_length(
            &self,
            accent_phrases: &[AccentPhrase],
            style_id: StyleId,
        ) -> Result<Vec<AccentPhrase>> {
            let (_, phoneme_data_list) = initial_process(accent_phrases);

            let (_, _, vowel_indexes_data) = split_mora(&phoneme_data_list);

            let phoneme_list_s: Vec<i64> = phoneme_data_list
                .iter()
                .map(|phoneme_data| phoneme_data.phoneme_id())
                .collect();
            let phoneme_length = self.predict_duration(&phoneme_list_s, style_id).await?;

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
                                vowel_length: phoneme_length
                                    [vowel_indexes_data[index + 1] as usize],
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

        pub(super) async fn replace_mora_pitch(
            &self,
            accent_phrases: &[AccentPhrase],
            style_id: StyleId,
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
                    style_id,
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

        pub(super) async fn audio_query_from_kana(
            &self,
            kana: &str,
            style_id: StyleId,
        ) -> Result<AudioQuery> {
            let accent_phrases = self.create_accent_phrases_from_kana(kana, style_id).await?;
            Ok(AudioQuery::from_accent_phrases(accent_phrases).with_kana(Some(kana.to_owned())))
        }

        pub(super) async fn tts_from_kana(
            &self,
            kana: &str,
            style_id: StyleId,
            options: &TtsOptions,
        ) -> Result<Vec<u8>> {
            let audio_query = &self.audio_query_from_kana(kana, style_id).await?;
            self.synthesis(audio_query, style_id, &SynthesisOptions::from(options))
                .await
        }
    }

    impl<O: FullcontextExtractor, A: AsyncForOnnxruntime> Inner<O, A> {
        pub(super) async fn create_accent_phrases(
            &self,
            text: &str,
            style_id: StyleId,
        ) -> Result<Vec<AccentPhrase>> {
            let accent_phrases = self.open_jtalk_analyzer.analyze(text)?;
            self.replace_mora_data(&accent_phrases, style_id).await
        }

        pub(super) async fn audio_query(
            &self,
            text: &str,
            style_id: StyleId,
        ) -> Result<AudioQuery> {
            let accent_phrases = self.create_accent_phrases(text, style_id).await?;
            Ok(AudioQuery::from_accent_phrases(accent_phrases))
        }

        pub(super) async fn tts(
            &self,
            text: &str,
            style_id: StyleId,
            options: &TtsOptions,
        ) -> Result<Vec<u8>> {
            let audio_query = &self.audio_query(text, style_id).await?;
            self.synthesis(audio_query, style_id, &SynthesisOptions::from(options))
                .await
        }
    }

    impl<O, A: AsyncForOnnxruntime> Inner<O, A> {
        pub(super) async fn predict_duration(
            &self,
            phoneme_vector: &[i64],
            style_id: StyleId,
        ) -> Result<Vec<f32>> {
            let status = self.status.clone();
            let phoneme_vector = ndarray::arr1(phoneme_vector);
            A::unblock(move || status.predict_duration(phoneme_vector, style_id)).await
        }

        #[expect(
            clippy::too_many_arguments,
            reason = "compatible_engineでの`predict_intonation`の形を考えると、ここの引数を構造体に\
                      まとめたりしても可読性に寄与しない"
        )]
        pub(super) async fn predict_intonation(
            &self,
            length: usize,
            vowel_phoneme_vector: &[i64],
            consonant_phoneme_vector: &[i64],
            start_accent_vector: &[i64],
            end_accent_vector: &[i64],
            start_accent_phrase_vector: &[i64],
            end_accent_phrase_vector: &[i64],
            style_id: StyleId,
        ) -> Result<Vec<f32>> {
            let status = self.status.clone();
            let vowel_phoneme_vector = ndarray::arr1(vowel_phoneme_vector);
            let consonant_phoneme_vector = ndarray::arr1(consonant_phoneme_vector);
            let start_accent_vector = ndarray::arr1(start_accent_vector);
            let end_accent_vector = ndarray::arr1(end_accent_vector);
            let start_accent_phrase_vector = ndarray::arr1(start_accent_phrase_vector);
            let end_accent_phrase_vector = ndarray::arr1(end_accent_phrase_vector);
            A::unblock(move || {
                status.predict_intonation(
                    length,
                    vowel_phoneme_vector,
                    consonant_phoneme_vector,
                    start_accent_vector,
                    end_accent_vector,
                    start_accent_phrase_vector,
                    end_accent_phrase_vector,
                    style_id,
                )
            })
            .await
        }

        pub(super) async fn generate_full_intermediate(
            &self,
            length: usize,
            phoneme_size: usize,
            f0: &[f32],
            phoneme_vector: &[f32],
            style_id: StyleId,
        ) -> Result<ndarray::Array2<f32>> {
            let status = self.status.clone();
            let f0 = ndarray::arr1(f0);
            let phoneme_vector = ndarray::arr1(phoneme_vector);
            A::unblock(move || {
                status.generate_full_intermediate(
                    length,
                    phoneme_size,
                    f0,
                    phoneme_vector,
                    style_id,
                )
            })
            .await
        }

        pub(super) async fn render_audio_segment(
            &self,
            spec: ndarray::Array2<f32>,
            style_id: StyleId,
        ) -> Result<ndarray::Array1<f32>> {
            let status = self.status.clone();
            A::unblock(move || status.render_audio_segment(spec, style_id)).await
        }

        pub(super) async fn decode(
            &self,
            length: usize,
            phoneme_size: usize,
            f0: &[f32],
            phoneme_vector: &[f32],
            style_id: StyleId,
        ) -> Result<Vec<f32>> {
            let status = self.status.clone();
            let f0 = ndarray::arr1(f0);
            let phoneme_vector = ndarray::arr1(phoneme_vector);
            A::unblock(move || status.decode(length, phoneme_size, f0, phoneme_vector, style_id))
                .await
        }
    }

    impl<R: InferenceRuntime> Status<R> {
        /// CPU-boundな操作なので、非同期ランタイム上では直接実行されるべきではない。
        fn predict_duration(
            &self,
            phoneme_vector: ndarray::Array1<i64>,
            style_id: StyleId,
        ) -> Result<Vec<f32>> {
            let (model_id, inner_voice_id) = self.ids_for::<TalkDomain>(style_id)?;

            let PredictDurationOutput {
                phoneme_length: output,
            } = self.run_session(
                model_id,
                PredictDurationInput {
                    phoneme_list: phoneme_vector,
                    speaker_id: ndarray::arr1(&[inner_voice_id.raw_id().into()]),
                },
            )?;
            let mut output = output.into_raw_vec();

            for output_item in output.iter_mut() {
                if *output_item < PHONEME_LENGTH_MINIMAL {
                    *output_item = PHONEME_LENGTH_MINIMAL;
                }
            }

            return Ok(output);

            const PHONEME_LENGTH_MINIMAL: f32 = 0.01;
        }

        /// CPU-boundな操作なので、非同期ランタイム上では直接実行されるべきではない。
        #[expect(
            clippy::too_many_arguments,
            reason = "compatible_engineでの`predict_intonation`の形を考えると、ここの引数を構造体に\
                      まとめたりしても可読性に寄与しない"
        )]
        fn predict_intonation(
            &self,
            length: usize,
            vowel_phoneme_vector: ndarray::Array1<i64>,
            consonant_phoneme_vector: ndarray::Array1<i64>,
            start_accent_vector: ndarray::Array1<i64>,
            end_accent_vector: ndarray::Array1<i64>,
            start_accent_phrase_vector: ndarray::Array1<i64>,
            end_accent_phrase_vector: ndarray::Array1<i64>,
            style_id: StyleId,
        ) -> Result<Vec<f32>> {
            let (model_id, inner_voice_id) = self.ids_for::<TalkDomain>(style_id)?;

            let PredictIntonationOutput { f0_list: output } = self.run_session(
                model_id,
                PredictIntonationInput {
                    length: ndarray::arr0(length as i64),
                    vowel_phoneme_list: vowel_phoneme_vector,
                    consonant_phoneme_list: consonant_phoneme_vector,
                    start_accent_list: start_accent_vector,
                    end_accent_list: end_accent_vector,
                    start_accent_phrase_list: start_accent_phrase_vector,
                    end_accent_phrase_list: end_accent_phrase_vector,
                    speaker_id: ndarray::arr1(&[inner_voice_id.raw_id().into()]),
                },
            )?;

            Ok(output.into_raw_vec())
        }

        /// モデル`generate_full_intermediate`の実行と、その前後の処理を行う。
        ///
        /// 無音パディングを付加して音声特徴量を計算し、マージン込みの音声特徴量を返す。
        ///
        /// CPU-boundな操作なので、非同期ランタイム上では直接実行されるべきではない。
        fn generate_full_intermediate(
            &self,
            length: usize,
            phoneme_size: usize,
            f0: ndarray::Array1<f32>,
            phoneme_vector: ndarray::Array1<f32>,
            style_id: StyleId,
        ) -> Result<ndarray::Array2<f32>> {
            let (model_id, inner_voice_id) = self.ids_for::<TalkDomain>(style_id)?;

            // 音が途切れてしまうのを避けるworkaround処理が入っている
            // TODO: 改善したらここのpadding処理を取り除く
            let start_and_end_padding_size = 2 * PADDING_FRAME_LENGTH;
            let length_with_padding = f0.len() + start_and_end_padding_size;
            let f0_with_padding = make_f0_with_padding(f0, PADDING_FRAME_LENGTH);
            let phoneme_with_padding = make_phoneme_with_padding(
                phoneme_vector.into_shape([length, phoneme_size]).unwrap(),
                PADDING_FRAME_LENGTH,
            );

            let GenerateFullIntermediateOutput {
                spec: spec_with_padding,
            } = self.run_session(
                model_id,
                GenerateFullIntermediateInput {
                    f0: f0_with_padding
                        .into_shape([length_with_padding, 1])
                        .unwrap(),
                    phoneme: phoneme_with_padding,
                    speaker_id: ndarray::arr1(&[inner_voice_id.raw_id().into()]),
                },
            )?;

            // マージンがデータからはみ出さないことを保証
            // cf. https://github.com/VOICEVOX/voicevox_core/pull/854#discussion_r1803691291
            if MARGIN > PADDING_FRAME_LENGTH {
                unreachable!("Validation error: Too short padding for input, please report this issue on GitHub.");
            }
            // マージン分を両端に残して音声特徴量を返す
            return Ok(spec_with_padding
                .slice(ndarray::s![
                    PADDING_FRAME_LENGTH - MARGIN
                        ..spec_with_padding.nrows() - PADDING_FRAME_LENGTH + MARGIN,
                    ..
                ])
                .to_owned());

            fn make_f0_with_padding(
                f0_slice: ndarray::Array1<f32>,
                padding_size: usize,
            ) -> ndarray::Array1<f32> {
                // 音が途切れてしまうのを避けるworkaround処理
                // 改善したらこの関数を削除する
                let padding = ndarray::Array1::<f32>::zeros(padding_size);
                ndarray::concatenate![ndarray::Axis(0), padding, f0_slice, padding]
            }

            fn make_phoneme_with_padding(
                phoneme_slice: ndarray::Array2<f32>,
                padding_size: usize,
            ) -> ndarray::Array2<f32> {
                // 音が途切れてしまうのを避けるworkaround処理
                // 改善したらこの関数を削除する
                let mut padding =
                    ndarray::Array2::<f32>::zeros((padding_size, phoneme_slice.ncols()));
                padding
                    .slice_mut(ndarray::s![.., 0])
                    .assign(&ndarray::arr0(1.0));
                ndarray::concatenate![ndarray::Axis(0), padding, phoneme_slice, padding]
            }
        }

        /// 与えられた音声特徴量で音声生成。
        /// CPU/GPU-boundな操作なので、非同期ランタイム上では直接実行されるべきではない。
        fn render_audio_segment(
            &self,
            spec: ndarray::Array2<f32>,
            style_id: StyleId,
        ) -> Result<ndarray::Array1<f32>> {
            let (model_id, _inner_voice_id) = self.ids_for::<TalkDomain>(style_id)?;
            let RenderAudioSegmentOutput { wave } =
                self.run_session(model_id, RenderAudioSegmentInput { spec })?;
            Ok(wave)
        }

        /// CPU/GPU-boundな操作なので、非同期ランタイム上では直接実行されるべきではない。
        fn decode(
            &self,
            length: usize,
            phoneme_size: usize,
            f0: ndarray::Array1<f32>,
            phoneme_vector: ndarray::Array1<f32>,
            style_id: StyleId,
        ) -> Result<Vec<f32>> {
            let intermediate = self.generate_full_intermediate(
                length,
                phoneme_size,
                f0,
                phoneme_vector,
                style_id,
            )?;
            let output_with_margin = self.render_audio_segment(intermediate, style_id)?;
            let output = trim_margin_from_wave(output_with_margin);
            Ok(output.to_vec())
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

    fn initial_process(accent_phrases: &[AccentPhrase]) -> (Vec<Mora>, Vec<OjtPhoneme>) {
        let flatten_moras = to_flatten_moras(accent_phrases);

        let mut phoneme_strings = vec!["pau".to_string()];
        for mora in flatten_moras.iter() {
            if let Some(consonant) = &mora.consonant {
                phoneme_strings.push(consonant.clone())
            }
            phoneme_strings.push(mora.vowel.clone());
        }
        phoneme_strings.push("pau".to_string());

        let phoneme_data_list = to_phoneme_data_list(&phoneme_strings);

        return (flatten_moras, phoneme_data_list);

        fn to_flatten_moras(accent_phrases: &[AccentPhrase]) -> Vec<Mora> {
            let mut flatten_moras = Vec::new();

            for AccentPhrase {
                moras, pause_mora, ..
            } in accent_phrases
            {
                for mora in moras {
                    flatten_moras.push(mora.clone());
                }
                if let Some(pause_mora) = pause_mora {
                    flatten_moras.push(pause_mora.clone());
                }
            }

            flatten_moras
        }

        fn to_phoneme_data_list<T: AsRef<str>>(phoneme_str_list: &[T]) -> Vec<OjtPhoneme> {
            OjtPhoneme::convert(
                phoneme_str_list
                    .iter()
                    .map(AsRef::as_ref)
                    .map(ToOwned::to_owned)
                    .map(OjtPhoneme::new)
                    .collect::<Vec<OjtPhoneme>>()
                    .as_slice(),
            )
        }
    }

    fn split_mora(phoneme_list: &[OjtPhoneme]) -> (Vec<OjtPhoneme>, Vec<OjtPhoneme>, Vec<i64>) {
        let mut vowel_indexes = Vec::new();
        for (i, phoneme) in phoneme_list.iter().enumerate() {
            const MORA_PHONEME_LIST: &[&str] = &[
                "a", "i", "u", "e", "o", "N", "A", "I", "U", "E", "O", "cl", "pau",
            ];

            if MORA_PHONEME_LIST
                .iter()
                .any(|mora_phoneme| *mora_phoneme == phoneme.phoneme())
            {
                vowel_indexes.push(i as i64);
            }
        }

        let vowel_phoneme_list = vowel_indexes
            .iter()
            .map(|vowel_index| phoneme_list[*vowel_index as usize].clone())
            .collect();

        let mut consonant_phoneme_list = vec![OjtPhoneme::default()];
        for i in 0..(vowel_indexes.len() - 1) {
            let prev = vowel_indexes[i];
            let next = vowel_indexes[i + 1];
            if next - prev == 1 {
                consonant_phoneme_list.push(OjtPhoneme::default());
            } else {
                consonant_phoneme_list.push(phoneme_list[next as usize - 1].clone());
            }
        }

        (consonant_phoneme_list, vowel_phoneme_list, vowel_indexes)
    }

    impl AudioQuery {
        fn from_accent_phrases(accent_phrases: Vec<AccentPhrase>) -> Self {
            let kana = create_kana(&accent_phrases);
            Self {
                accent_phrases,
                speed_scale: 1.,
                pitch_scale: 0.,
                intonation_scale: 1.,
                volume_scale: 1.,
                pre_phoneme_length: 0.1,
                post_phoneme_length: 0.1,
                output_sampling_rate: DEFAULT_SAMPLING_RATE,
                output_stereo: false,
                pause_length: (),
                pause_length_scale: (),
                kana: Some(kana),
            }
        }
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "`PerformInference::predict_intonation`用。compatible_engineでの`predict_intonation`の\
              形を考えると、ここの引数を構造体にまとめたりしても可読性に寄与しない"
)]
pub(crate) mod blocking {
    use std::ops::Range;

    use easy_ext::ext;

    use crate::{
        asyncs::SingleTasked, future::FutureExt as _, AccentPhrase, AudioQuery,
        FullcontextExtractor, StyleId, VoiceModelId, VoiceModelMeta,
    };

    use super::{inner::Inner, InitializeOptions, SynthesisOptions, TtsOptions};

    pub use super::inner::AudioFeature;

    /// 音声シンセサイザ。
    pub struct Synthesizer<O>(pub(super) Inner<O, SingleTasked>);

    impl<O> self::Synthesizer<O> {
        /// `Synthesizer`をコンストラクトする。
        ///
        /// # Example
        ///
        #[cfg_attr(feature = "load-onnxruntime", doc = "```")]
        #[cfg_attr(not(feature = "load-onnxruntime"), doc = "```compile_fail")]
        /// # fn main() -> anyhow::Result<()> {
        /// # use test_util::{ONNXRUNTIME_DYLIB_PATH, OPEN_JTALK_DIC_DIR};
        /// #
        /// # const ACCELERATION_MODE: AccelerationMode = AccelerationMode::Cpu;
        /// #
        /// use std::sync::Arc;
        ///
        /// use voicevox_core::{
        ///     blocking::{Onnxruntime, OpenJtalk, Synthesizer},
        ///     AccelerationMode, InitializeOptions,
        /// };
        ///
        /// # if cfg!(windows) {
        /// #     // Windows\System32\onnxruntime.dllを回避
        /// #     voicevox_core::blocking::Onnxruntime::load_once()
        /// #         .filename(test_util::ONNXRUNTIME_DYLIB_PATH)
        /// #         .exec()?;
        /// # }
        /// let mut syntesizer = Synthesizer::new(
        ///     Onnxruntime::load_once().exec()?,
        ///     Arc::new(OpenJtalk::new(OPEN_JTALK_DIC_DIR).unwrap()),
        ///     &InitializeOptions {
        ///         acceleration_mode: ACCELERATION_MODE,
        ///         ..Default::default()
        ///     },
        /// )?;
        /// #
        /// # Ok(())
        /// # }
        /// ```
        pub fn new(
            onnxruntime: &'static crate::blocking::Onnxruntime,
            open_jtalk: O,
            options: &InitializeOptions,
        ) -> crate::Result<Self> {
            Inner::new(onnxruntime, open_jtalk, options).map(Self)
        }

        pub fn onnxruntime(&self) -> &'static crate::blocking::Onnxruntime {
            self.0.onnxruntime()
        }

        /// ハードウェアアクセラレーションがGPUモードか判定する。
        pub fn is_gpu_mode(&self) -> bool {
            self.0.is_gpu_mode()
        }

        /// 音声モデルを読み込む。
        pub fn load_voice_model(
            &self,
            model: &crate::blocking::VoiceModelFile,
        ) -> crate::Result<()> {
            self.0.load_voice_model(model.inner()).block_on()
        }

        /// 音声モデルの読み込みを解除する。
        pub fn unload_voice_model(&self, voice_model_id: VoiceModelId) -> crate::Result<()> {
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

        /// AudioQueryから音声合成用の中間表現を生成する。
        pub fn precompute_render(
            &self,
            audio_query: &AudioQuery,
            style_id: StyleId,
            options: &SynthesisOptions,
        ) -> crate::Result<AudioFeature> {
            self.0
                .precompute_render(audio_query, style_id, options)
                .block_on()
        }

        /// 中間表現から16bit PCMで音声波形を生成する。
        pub fn render(&self, audio: &AudioFeature, range: Range<usize>) -> crate::Result<Vec<u8>> {
            self.0.render(audio, range).block_on()
        }

        /// AudioQueryから直接WAVフォーマットで音声波形を生成する。
        pub fn synthesis(
            &self,
            audio_query: &AudioQuery,
            style_id: StyleId,
            options: &SynthesisOptions,
        ) -> crate::Result<Vec<u8>> {
            self.0.synthesis(audio_query, style_id, options).block_on()
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
        pub fn create_accent_phrases_from_kana(
            &self,
            kana: &str,
            style_id: StyleId,
        ) -> crate::Result<Vec<AccentPhrase>> {
            self.0
                .create_accent_phrases_from_kana(kana, style_id)
                .block_on()
        }

        /// AccentPhraseの配列の音高・音素長を、特定の声で生成しなおす。
        pub fn replace_mora_data(
            &self,
            accent_phrases: &[AccentPhrase],
            style_id: StyleId,
        ) -> crate::Result<Vec<AccentPhrase>> {
            self.0
                .replace_mora_data(accent_phrases, style_id)
                .block_on()
        }

        /// AccentPhraseの配列の音素長を、特定の声で生成しなおす。
        pub fn replace_phoneme_length(
            &self,
            accent_phrases: &[AccentPhrase],
            style_id: StyleId,
        ) -> crate::Result<Vec<AccentPhrase>> {
            self.0
                .replace_phoneme_length(accent_phrases, style_id)
                .block_on()
        }

        /// AccentPhraseの配列の音高を、特定の声で生成しなおす。
        pub fn replace_mora_pitch(
            &self,
            accent_phrases: &[AccentPhrase],
            style_id: StyleId,
        ) -> crate::Result<Vec<AccentPhrase>> {
            self.0
                .replace_mora_pitch(accent_phrases, style_id)
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
        /// let audio_query = synthesizer.audio_query_from_kana("コンニチワ'", StyleId::new(302))?;
        /// #
        /// # Ok(())
        /// # }
        /// ```
        ///
        /// [AudioQuery]: crate::AudioQuery
        pub fn audio_query_from_kana(
            &self,
            kana: &str,
            style_id: StyleId,
        ) -> crate::Result<AudioQuery> {
            self.0.audio_query_from_kana(kana, style_id).block_on()
        }

        /// AquesTalk風記法から音声合成を行う。
        pub fn tts_from_kana(
            &self,
            kana: &str,
            style_id: StyleId,
            options: &TtsOptions,
        ) -> crate::Result<Vec<u8>> {
            self.0.tts_from_kana(kana, style_id, options).block_on()
        }
    }

    impl<O: FullcontextExtractor> self::Synthesizer<O> {
        /// 日本語のテキストからAccentPhrase (アクセント句)の配列を生成する。
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
        pub fn create_accent_phrases(
            &self,
            text: &str,
            style_id: StyleId,
        ) -> crate::Result<Vec<AccentPhrase>> {
            self.0.create_accent_phrases(text, style_id).block_on()
        }

        /// 日本語のテキストから[AudioQuery]を生成する。
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
        /// let audio_query = synthesizer.audio_query("こんにちは", StyleId::new(302))?;
        /// #
        /// # Ok(())
        /// # }
        /// ```
        ///
        /// [AudioQuery]: crate::AudioQuery
        pub fn audio_query(&self, text: &str, style_id: StyleId) -> crate::Result<AudioQuery> {
            self.0.audio_query(text, style_id).block_on()
        }

        /// 日本語のテキストから音声合成を行う。
        pub fn tts(
            &self,
            text: &str,
            style_id: StyleId,
            options: &TtsOptions,
        ) -> crate::Result<Vec<u8>> {
            self.0.tts(text, style_id, options).block_on()
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
                .decode(length, phoneme_size, f0, phoneme_vector, style_id)
                .block_on()
        }
    }
}

pub(crate) mod nonblocking {
    use easy_ext::ext;

    use crate::{
        asyncs::BlockingThreadPool, AccentPhrase, AudioQuery, FullcontextExtractor, Result,
        StyleId, SynthesisOptions, VoiceModelId, VoiceModelMeta,
    };

    use super::{inner::Inner, InitializeOptions, TtsOptions};

    /// 音声シンセサイザ。
    ///
    /// # Performance
    ///
    /// [blocking]クレートにより動いている。詳しくは[`nonblocking`モジュールのドキュメント]を参照。
    ///
    /// [blocking]: https://docs.rs/crate/blocking
    /// [`nonblocking`モジュールのドキュメント]: crate::nonblocking
    pub struct Synthesizer<O>(pub(super) Inner<O, BlockingThreadPool>);

    impl<O: Send + Sync + 'static> self::Synthesizer<O> {
        /// `Synthesizer`をコンストラクトする。
        ///
        /// # Example
        ///
        #[cfg_attr(feature = "load-onnxruntime", doc = "```")]
        #[cfg_attr(not(feature = "load-onnxruntime"), doc = "```compile_fail")]
        /// # #[pollster::main]
        /// # async fn main() -> anyhow::Result<()> {
        /// # use test_util::{ONNXRUNTIME_DYLIB_PATH, OPEN_JTALK_DIC_DIR};
        /// #
        /// # const ACCELERATION_MODE: AccelerationMode = AccelerationMode::Cpu;
        /// #
        /// use std::sync::Arc;
        ///
        /// use voicevox_core::{
        ///     nonblocking::{Onnxruntime, OpenJtalk, Synthesizer},
        ///     AccelerationMode, InitializeOptions,
        /// };
        ///
        /// # if cfg!(windows) {
        /// #     // Windows\System32\onnxruntime.dllを回避
        /// #     voicevox_core::blocking::Onnxruntime::load_once()
        /// #         .filename(test_util::ONNXRUNTIME_DYLIB_PATH)
        /// #         .exec()?;
        /// # }
        /// let mut syntesizer = Synthesizer::new(
        ///     Onnxruntime::load_once().exec().await?,
        ///     Arc::new(OpenJtalk::new(OPEN_JTALK_DIC_DIR).await.unwrap()),
        ///     &InitializeOptions {
        ///         acceleration_mode: ACCELERATION_MODE,
        ///         ..Default::default()
        ///     },
        /// )?;
        /// #
        /// # Ok(())
        /// # }
        /// ```
        pub fn new(
            onnxruntime: &'static crate::nonblocking::Onnxruntime,
            open_jtalk: O,
            options: &InitializeOptions,
        ) -> Result<Self> {
            Inner::new(&onnxruntime.0, open_jtalk, options)
                .map(Into::into)
                .map(Self)
        }

        pub fn onnxruntime(&self) -> &'static crate::nonblocking::Onnxruntime {
            crate::nonblocking::Onnxruntime::from_blocking(self.0.onnxruntime())
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
        pub async fn synthesis(
            &self,
            audio_query: &AudioQuery,
            style_id: StyleId,
            options: &SynthesisOptions,
        ) -> Result<Vec<u8>> {
            self.0.synthesis(audio_query, style_id, options).await
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
            style_id: StyleId,
        ) -> Result<Vec<AccentPhrase>> {
            self.0.create_accent_phrases_from_kana(kana, style_id).await
        }

        /// AccentPhraseの配列の音高・音素長を、特定の声で生成しなおす。
        pub async fn replace_mora_data(
            &self,
            accent_phrases: &[AccentPhrase],
            style_id: StyleId,
        ) -> Result<Vec<AccentPhrase>> {
            self.0.replace_mora_data(accent_phrases, style_id).await
        }

        /// AccentPhraseの配列の音素長を、特定の声で生成しなおす。
        pub async fn replace_phoneme_length(
            &self,
            accent_phrases: &[AccentPhrase],
            style_id: StyleId,
        ) -> Result<Vec<AccentPhrase>> {
            self.0
                .replace_phoneme_length(accent_phrases, style_id)
                .await
        }

        /// AccentPhraseの配列の音高を、特定の声で生成しなおす。
        pub async fn replace_mora_pitch(
            &self,
            accent_phrases: &[AccentPhrase],
            style_id: StyleId,
        ) -> Result<Vec<AccentPhrase>> {
            self.0.replace_mora_pitch(accent_phrases, style_id).await
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
        ///     .audio_query_from_kana("コンニチワ'", StyleId::new(302))
        ///     .await?;
        /// #
        /// # Ok(())
        /// # }
        /// ```
        ///
        /// [AudioQuery]: crate::AudioQuery
        pub async fn audio_query_from_kana(
            &self,
            kana: &str,
            style_id: StyleId,
        ) -> Result<AudioQuery> {
            self.0.audio_query_from_kana(kana, style_id).await
        }

        /// AquesTalk風記法から音声合成を行う。
        pub async fn tts_from_kana(
            &self,
            kana: &str,
            style_id: StyleId,
            options: &TtsOptions,
        ) -> Result<Vec<u8>> {
            self.0.tts_from_kana(kana, style_id, options).await
        }
    }

    impl<T: FullcontextExtractor> self::Synthesizer<T> {
        /// 日本語のテキストからAccentPhrase (アクセント句)の配列を生成する。
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
        pub async fn create_accent_phrases(
            &self,
            text: &str,
            style_id: StyleId,
        ) -> Result<Vec<AccentPhrase>> {
            self.0.create_accent_phrases(text, style_id).await
        }

        /// 日本語のテキストから[AudioQuery]を生成する。
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
        ///     .audio_query("こんにちは", StyleId::new(302))
        ///     .await?;
        /// #
        /// # Ok(())
        /// # }
        /// ```
        ///
        /// [AudioQuery]: crate::AudioQuery
        pub async fn audio_query(&self, text: &str, style_id: StyleId) -> Result<AudioQuery> {
            self.0.audio_query(text, style_id).await
        }

        /// 日本語のテキストから音声合成を行う。
        pub async fn tts(
            &self,
            text: &str,
            style_id: StyleId,
            options: &TtsOptions,
        ) -> Result<Vec<u8>> {
            self.0.tts(text, style_id, options).await
        }
    }

    #[ext(IntoBlocking)]
    impl<O> self::Synthesizer<O> {
        pub fn into_blocking(self) -> super::blocking::Synthesizer<O> {
            super::blocking::Synthesizer(self.0.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AccelerationMode, InitializeOptions};
    use crate::{engine::Mora, macros::tests::assert_debug_fmt_eq, AccentPhrase, Result, StyleId};
    use ::test_util::OPEN_JTALK_DIC_DIR;
    use rstest::rstest;

    #[rstest]
    #[case(Ok(()))]
    #[tokio::test]
    async fn load_model_works(#[case] expected_result_at_initialized: Result<()>) {
        let syntesizer = super::nonblocking::Synthesizer::new(
            crate::nonblocking::Onnxruntime::from_test_util_data()
                .await
                .unwrap(),
            (),
            &InitializeOptions {
                acceleration_mode: AccelerationMode::Cpu,
                ..Default::default()
            },
        )
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
        let syntesizer = super::nonblocking::Synthesizer::new(
            crate::nonblocking::Onnxruntime::from_test_util_data()
                .await
                .unwrap(),
            (),
            &InitializeOptions {
                acceleration_mode: AccelerationMode::Cpu,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(!syntesizer.is_gpu_mode());
    }

    #[rstest]
    #[case(1, true)]
    #[tokio::test]
    async fn is_loaded_model_by_style_id_works(#[case] style_id: u32, #[case] expected: bool) {
        let style_id = StyleId::new(style_id);
        let syntesizer = super::nonblocking::Synthesizer::new(
            crate::nonblocking::Onnxruntime::from_test_util_data()
                .await
                .unwrap(),
            (),
            &InitializeOptions {
                acceleration_mode: AccelerationMode::Cpu,
                ..Default::default()
            },
        )
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
        let syntesizer = super::nonblocking::Synthesizer::new(
            crate::nonblocking::Onnxruntime::from_test_util_data()
                .await
                .unwrap(),
            (),
            &InitializeOptions {
                acceleration_mode: AccelerationMode::Cpu,
                ..Default::default()
            },
        )
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
        let syntesizer = super::nonblocking::Synthesizer::new(
            crate::nonblocking::Onnxruntime::from_test_util_data()
                .await
                .unwrap(),
            (),
            &InitializeOptions {
                acceleration_mode: AccelerationMode::Cpu,
                ..Default::default()
            },
        )
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
        let syntesizer = super::nonblocking::Synthesizer::new(
            crate::nonblocking::Onnxruntime::from_test_util_data()
                .await
                .unwrap(),
            (),
            &InitializeOptions {
                acceleration_mode: AccelerationMode::Cpu,
                ..Default::default()
            },
        )
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
            .decode(F0_LENGTH, PHONEME_SIZE, &f0, &phoneme, StyleId::new(1))
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
    async fn audio_query_works(
        #[case] input: Input,
        #[case] expected_text_consonant_vowel_data: &TextConsonantVowelData,
        #[case] expected_kana_text: &str,
    ) {
        let syntesizer = super::nonblocking::Synthesizer::new(
            crate::nonblocking::Onnxruntime::from_test_util_data()
                .await
                .unwrap(),
            crate::nonblocking::OpenJtalk::new(OPEN_JTALK_DIC_DIR)
                .await
                .unwrap(),
            &InitializeOptions {
                acceleration_mode: AccelerationMode::Cpu,
                ..Default::default()
            },
        )
        .unwrap();

        let model = &crate::nonblocking::VoiceModelFile::sample().await.unwrap();
        syntesizer.load_voice_model(model).await.unwrap();

        let query = match input {
            Input::Kana(input) => {
                syntesizer
                    .audio_query_from_kana(input, StyleId::new(0))
                    .await
            }
            Input::Japanese(input) => syntesizer.audio_query(input, StyleId::new(0)).await,
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
        let syntesizer = super::nonblocking::Synthesizer::new(
            crate::nonblocking::Onnxruntime::from_test_util_data()
                .await
                .unwrap(),
            crate::nonblocking::OpenJtalk::new(OPEN_JTALK_DIC_DIR)
                .await
                .unwrap(),
            &InitializeOptions {
                acceleration_mode: AccelerationMode::Cpu,
                ..Default::default()
            },
        )
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
        let syntesizer = super::nonblocking::Synthesizer::new(
            crate::nonblocking::Onnxruntime::from_test_util_data()
                .await
                .unwrap(),
            crate::nonblocking::OpenJtalk::new(OPEN_JTALK_DIC_DIR)
                .await
                .unwrap(),
            &InitializeOptions {
                acceleration_mode: AccelerationMode::Cpu,
                ..Default::default()
            },
        )
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
        let syntesizer = super::nonblocking::Synthesizer::new(
            crate::nonblocking::Onnxruntime::from_test_util_data()
                .await
                .unwrap(),
            crate::nonblocking::OpenJtalk::new(OPEN_JTALK_DIC_DIR)
                .await
                .unwrap(),
            &InitializeOptions {
                acceleration_mode: AccelerationMode::Cpu,
                ..Default::default()
            },
        )
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
        let syntesizer = super::nonblocking::Synthesizer::new(
            crate::nonblocking::Onnxruntime::from_test_util_data()
                .await
                .unwrap(),
            crate::nonblocking::OpenJtalk::new(OPEN_JTALK_DIC_DIR)
                .await
                .unwrap(),
            &InitializeOptions {
                acceleration_mode: AccelerationMode::Cpu,
                ..Default::default()
            },
        )
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
        let syntesizer = super::nonblocking::Synthesizer::new(
            crate::nonblocking::Onnxruntime::from_test_util_data()
                .await
                .unwrap(),
            crate::nonblocking::OpenJtalk::new(OPEN_JTALK_DIC_DIR)
                .await
                .unwrap(),
            &InitializeOptions {
                acceleration_mode: AccelerationMode::Cpu,
                ..Default::default()
            },
        )
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
