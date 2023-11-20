use std::{
    borrow::Borrow,
    io::{Cursor, Write as _},
    sync::Arc,
};

use ndarray::{Array1, ArrayView, ArrayView1, ArrayView2};

use crate::{
    engine::{
        self, create_kana, parse_kana, AccentPhraseModel, MoraModel, OjtPhoneme, OpenJtalk,
        Utterance,
    },
    infer::runtimes::Onnxruntime,
    numerics::F32Ext as _,
};

use super::*;

/// [`Synthesizer::synthesis`]のオプション。
///
/// [`Synthesizer::synthesis`]: Synthesizer::synthesis
#[derive(Clone, Copy)]
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

/// [`Synthesizer::tts`]のオプション。
///
/// [`Synthesizer::tts`]: Synthesizer::tts
#[derive(Clone, Copy)]
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
#[derive(Default, Debug, PartialEq, Eq)]
pub enum AccelerationMode {
    /// 実行環境に合った適切なハードウェアアクセラレーションモードを選択する。
    #[default]
    Auto,
    /// ハードウェアアクセラレーションモードを"CPU"に設定する。
    Cpu,
    /// ハードウェアアクセラレーションモードを"GPU"に設定する。
    Gpu,
}

/// [`Synthesizer::new`]のオプション。
///
/// [`Synthesizer::new`]: Synthesizer::new
#[derive(Default)]
pub struct InitializeOptions {
    pub acceleration_mode: AccelerationMode,
    pub cpu_num_threads: u16,
}

pub(crate) type InferenceRuntimeImpl = Onnxruntime;

/// 音声シンセサイザ。
pub struct Synthesizer<O>(Arc<Inner<O>>);

impl<O: Send + Sync + 'static> Synthesizer<O> {
    /// `Synthesizer`をコンストラクトする。
    ///
    /// # Example
    ///
    #[cfg_attr(windows, doc = "```no_run")] // https://github.com/VOICEVOX/voicevox_core/issues/537
    #[cfg_attr(not(windows), doc = "```")]
    /// # fn main() -> anyhow::Result<()> {
    /// # use test_util::OPEN_JTALK_DIC_DIR;
    /// #
    /// # const ACCELERATION_MODE: AccelerationMode = AccelerationMode::Cpu;
    /// #
    /// use std::sync::Arc;
    ///
    /// use voicevox_core::{AccelerationMode, InitializeOptions, OpenJtalk, Synthesizer};
    ///
    /// let mut syntesizer = Synthesizer::new(
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
    pub fn new(open_jtalk: O, options: &InitializeOptions) -> Result<Self> {
        #[cfg(windows)]
        list_windows_video_cards();
        let use_gpu = match options.acceleration_mode {
            AccelerationMode::Auto => {
                let supported_devices = SupportedDevices::create()?;

                cfg_if! {
                    if #[cfg(feature="directml")]{
                        *supported_devices.dml()

                    } else {
                        *supported_devices.cuda()
                    }
                }
            }
            AccelerationMode::Cpu => false,
            AccelerationMode::Gpu => true,
        };

        Ok(Self(
            Inner {
                inference_core: InferenceCore::new(use_gpu, options.cpu_num_threads)?,
                open_jtalk,
            }
            .into(),
        ))
    }

    /// ハードウェアアクセラレーションがGPUモードか判定する。
    pub fn is_gpu_mode(&self) -> bool {
        self.0.inference_core.is_use_gpu()
    }

    /// 音声モデルを読み込む。
    pub async fn load_voice_model(&self, model: &VoiceModel) -> Result<()> {
        self.0.inference_core.load_model(model).await?;
        Ok(())
    }

    /// 音声モデルの読み込みを解除する。
    pub fn unload_voice_model(&self, voice_model_id: &VoiceModelId) -> Result<()> {
        self.0.inference_core.unload_model(voice_model_id)
    }

    /// 指定したIDの音声モデルが読み込まれているか判定する。
    pub fn is_loaded_voice_model(&self, voice_model_id: &VoiceModelId) -> bool {
        self.0.inference_core.is_loaded_model(voice_model_id)
    }

    #[doc(hidden)]
    pub fn is_loaded_model_by_style_id(&self, style_id: StyleId) -> bool {
        self.0.inference_core.is_model_loaded_by_style_id(style_id)
    }

    /// 今読み込んでいる音声モデルのメタ情報を返す。
    pub fn metas(&self) -> VoiceModelMeta {
        self.0.inference_core.metas()
    }

    /// AudioQueryから音声合成を行う。
    pub async fn synthesis(
        &self,
        audio_query: &AudioQueryModel,
        style_id: StyleId,
        options: &SynthesisOptions,
    ) -> Result<Vec<u8>> {
        let audio_query = audio_query.clone();
        let options = *options;

        self.spawn_blocking(move |inner| inner.blocking_synthesis(&audio_query, style_id, &options))
            .await
    }

    pub(crate) fn predict_duration(
        &self,
        phoneme_list: Array1<i64>,
        style_id: StyleId,
    ) -> Result<Vec<f32>> {
        self.0
            .inference_core
            .predict_duration(phoneme_list, style_id)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn predict_intonation(
        &self,
        vowel_phoneme_list: Array1<i64>,
        consonant_phoneme_list: Array1<i64>,
        start_accent_list: Array1<i64>,
        end_accent_list: Array1<i64>,
        start_accent_phrase_list: Array1<i64>,
        end_accent_phrase_list: Array1<i64>,
        style_id: StyleId,
    ) -> Result<Vec<f32>> {
        self.0.inference_core.predict_intonation(
            vowel_phoneme_list,
            consonant_phoneme_list,
            start_accent_list,
            end_accent_list,
            start_accent_phrase_list,
            end_accent_phrase_list,
            style_id,
        )
    }

    pub(crate) fn decode(
        &self,
        f0: ArrayView1<'_, f32>,
        phoneme: ArrayView2<'_, f32>,
        style_id: StyleId,
    ) -> Result<Vec<f32>> {
        self.0.inference_core.decode(f0, phoneme, style_id)
    }

    /// AquesTalk風記法からAccentPhrase (アクセント句)の配列を生成する。
    ///
    /// # Example
    ///
    #[cfg_attr(windows, doc = "```no_run")] // https://github.com/VOICEVOX/voicevox_core/issues/537
    #[cfg_attr(not(windows), doc = "```")]
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// # let synthesizer =
    /// #     voicevox_core::__internal::doctest_fixtures::synthesizer_with_sample_voice_model(
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
    ) -> Result<Vec<AccentPhraseModel>> {
        let kana = kana.to_owned();

        self.spawn_blocking(move |inner| {
            inner.blocking_create_accent_phrases_from_kana(&kana, style_id)
        })
        .await
    }

    /// AccentPhraseの配列の音高・音素長を、特定の声で生成しなおす。
    pub async fn replace_mora_data(
        &self,
        accent_phrases: &[AccentPhraseModel],
        style_id: StyleId,
    ) -> Result<Vec<AccentPhraseModel>> {
        let accent_phrases = accent_phrases.to_owned();

        self.spawn_blocking(move |inner| {
            inner.blocking_replace_mora_data(&accent_phrases, style_id)
        })
        .await
    }

    /// AccentPhraseの配列の音素長を、特定の声で生成しなおす。
    pub async fn replace_phoneme_length(
        &self,
        accent_phrases: &[AccentPhraseModel],
        style_id: StyleId,
    ) -> Result<Vec<AccentPhraseModel>> {
        let accent_phrases = accent_phrases.to_owned();

        self.spawn_blocking(move |inner| {
            inner.blocking_replace_phoneme_length(&accent_phrases, style_id)
        })
        .await
    }

    /// AccentPhraseの配列の音高を、特定の声で生成しなおす。
    pub async fn replace_mora_pitch(
        &self,
        accent_phrases: &[AccentPhraseModel],
        style_id: StyleId,
    ) -> Result<Vec<AccentPhraseModel>> {
        let accent_phrases = accent_phrases.to_owned();

        self.spawn_blocking(move |inner| {
            inner.blocking_replace_mora_pitch(&accent_phrases, style_id)
        })
        .await
    }

    /// AquesTalk風記法から[AudioQuery]を生成する。
    ///
    /// # Example
    ///
    #[cfg_attr(windows, doc = "```no_run")] // https://github.com/VOICEVOX/voicevox_core/issues/537
    #[cfg_attr(not(windows), doc = "```")]
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// # let synthesizer =
    /// #     voicevox_core::__internal::doctest_fixtures::synthesizer_with_sample_voice_model(
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
    /// [AudioQuery]: crate::AudioQueryModel
    pub async fn audio_query_from_kana(
        &self,
        kana: &str,
        style_id: StyleId,
    ) -> Result<AudioQueryModel> {
        let kana = kana.to_owned();

        self.spawn_blocking(move |inner| inner.blocking_audio_query_from_kana(&kana, style_id))
            .await
    }

    /// AquesTalk風記法から音声合成を行う。
    pub async fn tts_from_kana(
        &self,
        kana: &str,
        style_id: StyleId,
        options: &TtsOptions,
    ) -> Result<Vec<u8>> {
        let kana = kana.to_owned();
        let options = *options;

        self.spawn_blocking(move |inner| inner.blocking_tts_from_kana(&kana, style_id, &options))
            .await
    }

    async fn spawn_blocking<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&Inner<O>) -> Result<R> + Send + 'static,
        R: Send + 'static,
    {
        let inner = self.0.clone();

        tokio::task::spawn_blocking(move || f(&inner))
            .await
            .unwrap()
    }
}

impl<O: Borrow<OpenJtalk> + Send + Sync + 'static> Synthesizer<O> {
    /// 日本語のテキストからAccentPhrase (アクセント句)の配列を生成する。
    ///
    /// # Example
    ///
    #[cfg_attr(windows, doc = "```no_run")] // https://github.com/VOICEVOX/voicevox_core/issues/537
    #[cfg_attr(not(windows), doc = "```")]
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// # let synthesizer =
    /// #     voicevox_core::__internal::doctest_fixtures::synthesizer_with_sample_voice_model(
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
    ) -> Result<Vec<AccentPhraseModel>> {
        let text = text.to_owned();

        self.spawn_blocking(move |inner| inner.blocking_create_accent_phrases(&text, style_id))
            .await
    }

    /// 日本語のテキストから[AudioQuery]を生成する。
    ///
    /// # Examples
    ///
    #[cfg_attr(windows, doc = "```no_run")] // https://github.com/VOICEVOX/voicevox_core/issues/537
    #[cfg_attr(not(windows), doc = "```")]
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// # let synthesizer =
    /// #     voicevox_core::__internal::doctest_fixtures::synthesizer_with_sample_voice_model(
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
    /// [AudioQuery]: crate::AudioQueryModel
    pub async fn audio_query(&self, text: &str, style_id: StyleId) -> Result<AudioQueryModel> {
        let text = text.to_owned();

        self.spawn_blocking(move |inner| inner.blocking_audio_query(&text, style_id))
            .await
    }

    /// 日本語のテキストから音声合成を行う。
    pub async fn tts(
        &self,
        text: &str,
        style_id: StyleId,
        options: &TtsOptions,
    ) -> Result<Vec<u8>> {
        let text = text.to_owned();
        let options = *options;

        self.spawn_blocking(move |inner| inner.blocking_tts(&text, style_id, &options))
            .await
    }
}

struct Inner<O> {
    inference_core: InferenceCore<InferenceRuntimeImpl>,
    open_jtalk: O,
}

impl<O: Send + Sync> Inner<O> {
    fn blocking_synthesis(
        &self,
        audio_query: &AudioQueryModel,
        style_id: StyleId,
        options: &SynthesisOptions,
    ) -> Result<Vec<u8>> {
        let speed_scale = *audio_query.speed_scale();
        let pitch_scale = *audio_query.pitch_scale();
        let intonation_scale = *audio_query.intonation_scale();
        let pre_phoneme_length = *audio_query.pre_phoneme_length();
        let post_phoneme_length = *audio_query.post_phoneme_length();

        let accent_phrases = if options.enable_interrogative_upspeak {
            adjust_interrogative_accent_phrases(audio_query.accent_phrases().as_slice())
        } else {
            audio_query.accent_phrases().clone()
        };

        let (flatten_moras, phoneme_data_list) = initial_process(&accent_phrases);

        let mut phoneme_length_list = vec![pre_phoneme_length];
        let mut f0_list = vec![0.];
        let mut voiced_list = vec![false];
        {
            let mut sum_of_f0_bigger_than_zero = 0.;
            let mut count_of_f0_bigger_than_zero = 0;

            for mora in flatten_moras {
                let consonant_length = *mora.consonant_length();
                let vowel_length = *mora.vowel_length();
                let pitch = *mora.pitch();

                if let Some(consonant_length) = consonant_length {
                    phoneme_length_list.push(consonant_length);
                }
                phoneme_length_list.push(vowel_length);

                let f0_single = pitch * 2.0_f32.powf(pitch_scale);
                f0_list.push(f0_single);

                let bigger_than_zero = f0_single > 0.;
                voiced_list.push(bigger_than_zero);

                if bigger_than_zero {
                    sum_of_f0_bigger_than_zero += f0_single;
                    count_of_f0_bigger_than_zero += 1;
                }
            }
            phoneme_length_list.push(post_phoneme_length);
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

        let mut phoneme: Vec<[f32; OjtPhoneme::NUM_PHONEME]> = Vec::new();
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
                let phoneme_length = ((*phoneme_length * RATE).round_ties_even_() / speed_scale)
                    .round_ties_even_() as usize;
                let phoneme_id = phoneme_data_list[i].phoneme_id();

                for _ in 0..phoneme_length {
                    let mut phonemes_vec = [0.; OjtPhoneme::NUM_PHONEME];
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

        let phoneme = &phoneme.into_iter().flatten().collect::<Vec<_>>();
        let phoneme = ArrayView::from_shape((f0.len(), OjtPhoneme::NUM_PHONEME), phoneme).unwrap();

        let wave = &self
            .inference_core
            .decode(ndarray::aview1(&f0), phoneme, style_id)?;

        return Ok(to_wav(wave, audio_query));

        fn adjust_interrogative_accent_phrases(
            accent_phrases: &[AccentPhraseModel],
        ) -> Vec<AccentPhraseModel> {
            accent_phrases
                .iter()
                .map(|accent_phrase| {
                    AccentPhraseModel::new(
                        adjust_interrogative_moras(accent_phrase),
                        *accent_phrase.accent(),
                        accent_phrase.pause_mora().clone(),
                        *accent_phrase.is_interrogative(),
                    )
                })
                .collect()
        }

        fn adjust_interrogative_moras(accent_phrase: &AccentPhraseModel) -> Vec<MoraModel> {
            let moras = accent_phrase.moras();
            if *accent_phrase.is_interrogative() && !moras.is_empty() {
                let last_mora = moras.last().unwrap();
                let last_mora_pitch = *last_mora.pitch();
                if last_mora_pitch != 0.0 {
                    let mut new_moras: Vec<MoraModel> = Vec::with_capacity(moras.len() + 1);
                    new_moras.extend_from_slice(moras.as_slice());
                    let interrogative_mora = make_interrogative_mora(last_mora);
                    new_moras.push(interrogative_mora);
                    return new_moras;
                }
            }
            moras.clone()
        }

        fn make_interrogative_mora(last_mora: &MoraModel) -> MoraModel {
            const FIX_VOWEL_LENGTH: f32 = 0.15;
            const ADJUST_PITCH: f32 = 0.3;
            const MAX_PITCH: f32 = 6.5;

            let pitch = (*last_mora.pitch() + ADJUST_PITCH).min(MAX_PITCH);

            MoraModel::new(
                mora_to_text(last_mora.vowel()),
                None,
                None,
                last_mora.vowel().clone(),
                FIX_VOWEL_LENGTH,
                pitch,
            )
        }

        fn to_wav(wave: &[f32], query: &AudioQueryModel) -> Vec<u8> {
            let volume_scale = *query.volume_scale();
            let output_stereo = *query.output_stereo();
            let output_sampling_rate = *query.output_sampling_rate();

            // TODO: 44.1kHzなどの対応

            let num_channels: u16 = if output_stereo { 2 } else { 1 };
            let bit_depth: u16 = 16;
            let repeat_count: u32 =
                (output_sampling_rate / DEFAULT_SAMPLING_RATE) * num_channels as u32;
            let block_size: u16 = bit_depth * num_channels / 8;

            let bytes_size = wave.len() as u32 * repeat_count * 2;
            let wave_size = bytes_size + 44;

            let buf: Vec<u8> = Vec::with_capacity(wave_size as usize);
            let mut cur = Cursor::new(buf);

            cur.write_all("RIFF".as_bytes()).unwrap();
            cur.write_all(&(wave_size - 8).to_le_bytes()).unwrap();
            cur.write_all("WAVEfmt ".as_bytes()).unwrap();
            cur.write_all(&16_u32.to_le_bytes()).unwrap(); // fmt header length
            cur.write_all(&1_u16.to_le_bytes()).unwrap(); //linear PCM
            cur.write_all(&num_channels.to_le_bytes()).unwrap();
            cur.write_all(&output_sampling_rate.to_le_bytes()).unwrap();

            let block_rate = output_sampling_rate * block_size as u32;

            cur.write_all(&block_rate.to_le_bytes()).unwrap();
            cur.write_all(&block_size.to_le_bytes()).unwrap();
            cur.write_all(&bit_depth.to_le_bytes()).unwrap();
            cur.write_all("data".as_bytes()).unwrap();
            cur.write_all(&bytes_size.to_le_bytes()).unwrap();

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

    fn blocking_create_accent_phrases_from_kana(
        &self,
        kana: &str,
        style_id: StyleId,
    ) -> Result<Vec<AccentPhraseModel>> {
        self.blocking_replace_mora_data(&parse_kana(kana)?, style_id)
    }

    fn blocking_replace_mora_data(
        &self,
        accent_phrases: &[AccentPhraseModel],
        style_id: StyleId,
    ) -> Result<Vec<AccentPhraseModel>> {
        let accent_phrases = self.blocking_replace_phoneme_length(accent_phrases, style_id)?;
        self.blocking_replace_mora_pitch(&accent_phrases, style_id)
    }

    fn blocking_replace_phoneme_length(
        &self,
        accent_phrases: &[AccentPhraseModel],
        style_id: StyleId,
    ) -> Result<Vec<AccentPhraseModel>> {
        let (_, phoneme_data_list) = initial_process(accent_phrases);

        let (_, _, vowel_indexes_data) = split_mora(&phoneme_data_list);

        let phoneme_list_s: Array1<i64> = phoneme_data_list
            .iter()
            .map(|phoneme_data| phoneme_data.phoneme_id())
            .collect();
        let phoneme_length = self
            .inference_core
            .predict_duration(phoneme_list_s, style_id)?;

        let mut index = 0;
        let new_accent_phrases = accent_phrases
            .iter()
            .map(|accent_phrase| {
                AccentPhraseModel::new(
                    accent_phrase
                        .moras()
                        .iter()
                        .map(|mora| {
                            let new_mora = MoraModel::new(
                                mora.text().clone(),
                                mora.consonant().clone(),
                                mora.consonant().as_ref().map(|_| {
                                    phoneme_length[vowel_indexes_data[index + 1] as usize - 1]
                                }),
                                mora.vowel().clone(),
                                phoneme_length[vowel_indexes_data[index + 1] as usize],
                                *mora.pitch(),
                            );
                            index += 1;
                            new_mora
                        })
                        .collect(),
                    *accent_phrase.accent(),
                    accent_phrase.pause_mora().as_ref().map(|pause_mora| {
                        let new_pause_mora = MoraModel::new(
                            pause_mora.text().clone(),
                            pause_mora.consonant().clone(),
                            *pause_mora.consonant_length(),
                            pause_mora.vowel().clone(),
                            phoneme_length[vowel_indexes_data[index + 1] as usize],
                            *pause_mora.pitch(),
                        );
                        index += 1;
                        new_pause_mora
                    }),
                    *accent_phrase.is_interrogative(),
                )
            })
            .collect();

        Ok(new_accent_phrases)
    }

    fn blocking_replace_mora_pitch(
        &self,
        accent_phrases: &[AccentPhraseModel],
        style_id: StyleId,
    ) -> Result<Vec<AccentPhraseModel>> {
        let (_, phoneme_data_list) = initial_process(accent_phrases);

        let mut base_start_accent_list = vec![0];
        let mut base_end_accent_list = vec![0];
        let mut base_start_accent_phrase_list = vec![0];
        let mut base_end_accent_phrase_list = vec![0];
        for accent_phrase in accent_phrases {
            let mut accent = usize::from(*accent_phrase.accent() != 1);
            create_one_accent_list(&mut base_start_accent_list, accent_phrase, accent as i32);

            accent = *accent_phrase.accent() - 1;
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

        let mut f0_list = self.inference_core.predict_intonation(
            vowel_phoneme_list.into(),
            consonant_phoneme_list.into(),
            start_accent_list.into(),
            end_accent_list.into(),
            start_accent_phrase_list.into(),
            end_accent_phrase_list.into(),
            style_id,
        )?;

        for i in 0..vowel_phoneme_data_list.len() {
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
            .map(|accent_phrase| {
                AccentPhraseModel::new(
                    accent_phrase
                        .moras()
                        .iter()
                        .map(|mora| {
                            let new_mora = MoraModel::new(
                                mora.text().clone(),
                                mora.consonant().clone(),
                                *mora.consonant_length(),
                                mora.vowel().clone(),
                                *mora.vowel_length(),
                                f0_list[index + 1],
                            );
                            index += 1;
                            new_mora
                        })
                        .collect(),
                    *accent_phrase.accent(),
                    accent_phrase.pause_mora().as_ref().map(|pause_mora| {
                        let new_pause_mora = MoraModel::new(
                            pause_mora.text().clone(),
                            pause_mora.consonant().clone(),
                            *pause_mora.consonant_length(),
                            pause_mora.vowel().clone(),
                            *pause_mora.vowel_length(),
                            f0_list[index + 1],
                        );
                        index += 1;
                        new_pause_mora
                    }),
                    *accent_phrase.is_interrogative(),
                )
            })
            .collect();

        return Ok(new_accent_phrases);

        const UNVOICED_MORA_PHONEME_LIST: &[&str] = &["A", "I", "U", "E", "O", "cl", "pau"];

        fn create_one_accent_list(
            accent_list: &mut Vec<i64>,
            accent_phrase: &AccentPhraseModel,
            point: i32,
        ) {
            let mut one_accent_list: Vec<i64> = Vec::new();

            for (i, mora) in accent_phrase.moras().iter().enumerate() {
                let value = (i as i32 == point
                    || (point < 0 && i == (accent_phrase.moras().len() as i32 + point) as usize))
                    .into();
                one_accent_list.push(value);
                if mora.consonant().is_some() {
                    one_accent_list.push(value);
                }
            }
            if accent_phrase.pause_mora().is_some() {
                one_accent_list.push(0);
            }
            accent_list.extend(one_accent_list)
        }
    }

    fn blocking_audio_query_from_kana(
        &self,
        kana: &str,
        style_id: StyleId,
    ) -> Result<AudioQueryModel> {
        let accent_phrases = self.blocking_create_accent_phrases_from_kana(kana, style_id)?;
        Ok(AudioQueryModel::from_accent_phrases(accent_phrases).with_kana(Some(kana.to_owned())))
    }

    fn blocking_tts_from_kana(
        &self,
        kana: &str,
        style_id: StyleId,
        options: &TtsOptions,
    ) -> Result<Vec<u8>> {
        let audio_query = &self.blocking_audio_query_from_kana(kana, style_id)?;
        self.blocking_synthesis(audio_query, style_id, &SynthesisOptions::from(options))
    }
}

impl<O: Borrow<OpenJtalk> + Send + Sync + 'static> Inner<O> {
    fn blocking_create_accent_phrases(
        &self,
        text: &str,
        style_id: StyleId,
    ) -> Result<Vec<AccentPhraseModel>> {
        if text.is_empty() {
            return Ok(Vec::new());
        }

        let utterance = Utterance::extract_full_context_label(self.open_jtalk.borrow(), text)?;

        let accent_phrases: Vec<AccentPhraseModel> = utterance
            .breath_groups()
            .iter()
            .enumerate()
            .fold(Vec::new(), |mut accum_vec, (i, breath_group)| {
                accum_vec.extend(breath_group.accent_phrases().iter().enumerate().map(
                    |(j, accent_phrase)| {
                        let moras = accent_phrase
                            .moras()
                            .iter()
                            .map(|mora| {
                                let mora_text = mora
                                    .phonemes()
                                    .iter()
                                    .map(|phoneme| phoneme.phoneme().to_string())
                                    .collect::<Vec<_>>()
                                    .join("");

                                let (consonant, consonant_length) =
                                    if let Some(consonant) = mora.consonant() {
                                        (Some(consonant.phoneme().to_string()), Some(0.))
                                    } else {
                                        (None, None)
                                    };

                                MoraModel::new(
                                    mora_to_text(mora_text),
                                    consonant,
                                    consonant_length,
                                    mora.vowel().phoneme().into(),
                                    0.,
                                    0.,
                                )
                            })
                            .collect();

                        let pause_mora = if i != utterance.breath_groups().len() - 1
                            && j == breath_group.accent_phrases().len() - 1
                        {
                            Some(MoraModel::new(
                                "、".into(),
                                None,
                                None,
                                "pau".into(),
                                0.,
                                0.,
                            ))
                        } else {
                            None
                        };

                        AccentPhraseModel::new(
                            moras,
                            *accent_phrase.accent(),
                            pause_mora,
                            *accent_phrase.is_interrogative(),
                        )
                    },
                ));

                accum_vec
            });

        self.blocking_replace_mora_data(&accent_phrases, style_id)
    }

    fn blocking_audio_query(&self, text: &str, style_id: StyleId) -> Result<AudioQueryModel> {
        let accent_phrases = self.blocking_create_accent_phrases(text, style_id)?;
        Ok(AudioQueryModel::from_accent_phrases(accent_phrases))
    }

    fn blocking_tts(&self, text: &str, style_id: StyleId, options: &TtsOptions) -> Result<Vec<u8>> {
        let audio_query = &self.blocking_audio_query(text, style_id)?;
        self.blocking_synthesis(audio_query, style_id, &SynthesisOptions::from(options))
    }
}

const DEFAULT_SAMPLING_RATE: u32 = 24000;

#[cfg(windows)]
fn list_windows_video_cards() {
    use std::{ffi::OsString, os::windows::ffi::OsStringExt as _};

    use humansize::BINARY;
    use tracing::{error, info};
    use windows::Win32::Graphics::Dxgi::{
        CreateDXGIFactory, IDXGIFactory, DXGI_ADAPTER_DESC, DXGI_ERROR_NOT_FOUND,
    };

    info!("検出されたGPU (DirectMLには1番目のGPUが使われます):");
    match list_windows_video_cards() {
        Ok(descs) => {
            for desc in descs {
                let description = OsString::from_wide(trim_nul(&desc.Description));
                let vram = humansize::format_size(desc.DedicatedVideoMemory, BINARY);
                info!("  - {description:?} ({vram})");
            }
        }
        Err(err) => error!("{err}"),
    }

    fn list_windows_video_cards() -> windows::core::Result<Vec<DXGI_ADAPTER_DESC>> {
        #[allow(unsafe_code)]
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

fn initial_process(accent_phrases: &[AccentPhraseModel]) -> (Vec<MoraModel>, Vec<OjtPhoneme>) {
    let flatten_moras = to_flatten_moras(accent_phrases);

    let mut phoneme_strings = vec!["pau".to_string()];
    for mora in flatten_moras.iter() {
        if let Some(consonant) = mora.consonant() {
            phoneme_strings.push(consonant.clone())
        }
        phoneme_strings.push(mora.vowel().clone());
    }
    phoneme_strings.push("pau".to_string());

    let phoneme_data_list = to_phoneme_data_list(&phoneme_strings);

    return (flatten_moras, phoneme_data_list);

    fn to_flatten_moras(accent_phrases: &[AccentPhraseModel]) -> Vec<MoraModel> {
        let mut flatten_moras = Vec::new();

        for accent_phrase in accent_phrases {
            let moras = accent_phrase.moras();
            for mora in moras {
                flatten_moras.push(mora.clone());
            }
            if let Some(pause_mora) = accent_phrase.pause_mora() {
                flatten_moras.push(pause_mora.clone());
            }
        }

        flatten_moras
    }

    fn to_phoneme_data_list<T: AsRef<str>>(phoneme_str_list: &[T]) -> Vec<OjtPhoneme> {
        OjtPhoneme::convert(
            phoneme_str_list
                .iter()
                .enumerate()
                .map(|(i, s)| OjtPhoneme::new(s.as_ref().to_string(), i as f32, i as f32 + 1.))
                .collect::<Vec<OjtPhoneme>>()
                .as_slice(),
        )
    }
}

fn split_mora(phoneme_list: &[OjtPhoneme]) -> (Vec<OjtPhoneme>, Vec<OjtPhoneme>, Vec<i64>) {
    let mut vowel_indexes = Vec::new();
    for (i, phoneme) in phoneme_list.iter().enumerate() {
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

    return (consonant_phoneme_list, vowel_phoneme_list, vowel_indexes);

    const MORA_PHONEME_LIST: &[&str] = &[
        "a", "i", "u", "e", "o", "N", "A", "I", "U", "E", "O", "cl", "pau",
    ];
}

fn mora_to_text(mora: impl AsRef<str>) -> String {
    let last_char = mora.as_ref().chars().last().unwrap();
    let mora = if ['A', 'I', 'U', 'E', 'O'].contains(&last_char) {
        format!(
            "{}{}",
            &mora.as_ref()[0..mora.as_ref().len() - 1],
            last_char.to_lowercase()
        )
    } else {
        mora.as_ref().to_string()
    };
    // もしカタカナに変換できなければ、引数で与えた文字列がそのまま返ってくる
    engine::mora2text(&mora).to_string()
}

impl AudioQueryModel {
    fn from_accent_phrases(accent_phrases: Vec<AccentPhraseModel>) -> Self {
        let kana = create_kana(&accent_phrases);
        Self::new(
            accent_phrases,
            1.,
            0.,
            1.,
            1.,
            0.1,
            0.1,
            DEFAULT_SAMPLING_RATE,
            false,
            Some(kana),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::{engine::MoraModel, macros::tests::assert_debug_fmt_eq};
    use ::test_util::OPEN_JTALK_DIC_DIR;
    use ndarray::{array, s, Array};

    #[rstest]
    #[case(Ok(()))]
    #[tokio::test]
    async fn load_model_works(#[case] expected_result_at_initialized: Result<()>) {
        let syntesizer = Synthesizer::new(
            (),
            &InitializeOptions {
                acceleration_mode: AccelerationMode::Cpu,
                ..Default::default()
            },
        )
        .unwrap();

        let result = syntesizer
            .load_voice_model(&open_default_vvm_file().await)
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
        let syntesizer = Synthesizer::new(
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
        let syntesizer = Synthesizer::new(
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
            .load_voice_model(&open_default_vvm_file().await)
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
        let syntesizer = Synthesizer::new(
            (),
            &InitializeOptions {
                acceleration_mode: AccelerationMode::Cpu,
                ..Default::default()
            },
        )
        .unwrap();

        syntesizer
            .load_voice_model(&open_default_vvm_file().await)
            .await
            .unwrap();

        // 「こんにちは、音声合成の世界へようこそ」という文章を変換して得た phoneme_vector
        let phoneme_vector = array![
            0, 23, 30, 4, 28, 21, 10, 21, 42, 7, 0, 30, 4, 35, 14, 14, 16, 30, 30, 35, 14, 14, 28,
            30, 35, 14, 23, 7, 21, 14, 43, 30, 30, 23, 30, 35, 30, 0,
        ];

        let result = syntesizer.predict_duration(phoneme_vector.clone(), StyleId::new(1));

        assert!(result.is_ok(), "{result:?}");
        assert_eq!(result.unwrap().len(), phoneme_vector.len());
    }

    #[rstest]
    #[tokio::test]
    async fn predict_intonation_works() {
        let syntesizer = Synthesizer::new(
            (),
            &InitializeOptions {
                acceleration_mode: AccelerationMode::Cpu,
                ..Default::default()
            },
        )
        .unwrap();
        syntesizer
            .load_voice_model(&open_default_vvm_file().await)
            .await
            .unwrap();

        // 「テスト」という文章に対応する入力
        let vowel_phoneme_vector = array![0, 14, 6, 30, 0];
        let consonant_phoneme_vector = array![-1, 37, 35, 37, -1];
        let start_accent_vector = array![0, 1, 0, 0, 0];
        let end_accent_vector = array![0, 1, 0, 0, 0];
        let start_accent_phrase_vector = array![0, 1, 0, 0, 0];
        let end_accent_phrase_vector = array![0, 0, 0, 1, 0];

        let result = syntesizer.predict_intonation(
            vowel_phoneme_vector,
            consonant_phoneme_vector,
            start_accent_vector,
            end_accent_vector,
            start_accent_phrase_vector,
            end_accent_phrase_vector,
            StyleId::new(1),
        );

        assert!(result.is_ok(), "{result:?}");
        assert_eq!(result.unwrap().len(), 5);
    }

    #[rstest]
    #[tokio::test]
    async fn decode_works() {
        let syntesizer = Synthesizer::new(
            (),
            &InitializeOptions {
                acceleration_mode: AccelerationMode::Cpu,
                ..Default::default()
            },
        )
        .unwrap();
        syntesizer
            .load_voice_model(&open_default_vvm_file().await)
            .await
            .unwrap();

        // 「テスト」という文章に対応する入力
        const F0_LENGTH: usize = 69;
        let mut f0 = ndarray::arr1(&[0.; F0_LENGTH]);
        f0.slice_mut(s!(9..24)).fill(5.905218);
        f0.slice_mut(s!(37..60)).fill(5.565851);

        let mut phoneme = Array::from_shape_simple_fn((F0_LENGTH, OjtPhoneme::NUM_PHONEME), || 0.);
        let mut set_one = |index, range| {
            for i in range {
                phoneme[(i, index)] = 1.;
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

        let result = syntesizer.decode(f0.view(), phoneme.view(), StyleId::new(1));

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
        let syntesizer = Synthesizer::new(
            Arc::new(OpenJtalk::new(OPEN_JTALK_DIC_DIR).unwrap()),
            &InitializeOptions {
                acceleration_mode: AccelerationMode::Cpu,
                ..Default::default()
            },
        )
        .unwrap();

        let model = &VoiceModel::sample().await.unwrap();
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
            query.accent_phrases().len(),
            expected_text_consonant_vowel_data.len()
        );

        for (accent_phrase, (text_consonant_vowel_slice, accent_pos)) in
            std::iter::zip(query.accent_phrases(), expected_text_consonant_vowel_data)
        {
            assert_eq!(
                accent_phrase.moras().len(),
                text_consonant_vowel_slice.len()
            );
            assert_eq!(accent_phrase.accent(), accent_pos);

            for (mora, (text, consonant, vowel)) in
                std::iter::zip(accent_phrase.moras(), *text_consonant_vowel_slice)
            {
                assert_eq!(mora.text(), text);
                // NOTE: 子音の長さが必ず非ゼロになるテストケースを想定している
                assert_ne!(
                    mora.consonant_length(),
                    &Some(0.),
                    "expected mora.consonant_length is not Some(0.0), but got Some(0.0)."
                );
                assert_eq!(mora.consonant(), &Some(consonant.to_string()));
                assert_eq!(mora.vowel(), vowel);
                // NOTE: 母音の長さが必ず非ゼロになるテストケースを想定している
                assert_ne!(
                    mora.vowel_length(),
                    &0.,
                    "expected mora.vowel_length is not 0.0, but got 0.0."
                );
            }
        }

        assert_eq!(query.kana().as_deref(), Some(expected_kana_text));
    }

    #[rstest]
    #[case(Input::Japanese("これはテストです"), TEXT_CONSONANT_VOWEL_DATA1)]
    #[case(Input::Kana("コ'レワ/テ_スト'デ_ス"), TEXT_CONSONANT_VOWEL_DATA2)]
    #[tokio::test]
    async fn accent_phrases_works(
        #[case] input: Input,
        #[case] expected_text_consonant_vowel_data: &TextConsonantVowelData,
    ) {
        let syntesizer = Synthesizer::new(
            Arc::new(OpenJtalk::new(OPEN_JTALK_DIC_DIR).unwrap()),
            &InitializeOptions {
                acceleration_mode: AccelerationMode::Cpu,
                ..Default::default()
            },
        )
        .unwrap();

        let model = &VoiceModel::sample().await.unwrap();
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

        for (accent_phrase, (text_consonant_vowel_slice, accent_pos)) in
            std::iter::zip(accent_phrases, expected_text_consonant_vowel_data)
        {
            assert_eq!(
                accent_phrase.moras().len(),
                text_consonant_vowel_slice.len()
            );
            assert_eq!(accent_phrase.accent(), accent_pos);

            for (mora, (text, consonant, vowel)) in
                std::iter::zip(accent_phrase.moras(), *text_consonant_vowel_slice)
            {
                assert_eq!(mora.text(), text);
                // NOTE: 子音の長さが必ず非ゼロになるテストケースを想定している
                assert_ne!(
                    mora.consonant_length(),
                    &Some(0.),
                    "expected mora.consonant_length is not Some(0.0), but got Some(0.0)."
                );
                assert_eq!(mora.consonant(), &Some(consonant.to_string()));
                assert_eq!(mora.vowel(), vowel);
                // NOTE: 母音の長さが必ず非ゼロになるテストケースを想定している
                assert_ne!(
                    mora.vowel_length(),
                    &0.,
                    "expected mora.vowel_length is not 0.0, but got 0.0."
                );
            }
        }
    }

    #[rstest]
    #[tokio::test]
    async fn accent_phrases_works_for_japanese_periods_and_commas() {
        let syntesizer = Synthesizer::new(
            Arc::new(OpenJtalk::new(OPEN_JTALK_DIC_DIR).unwrap()),
            &InitializeOptions {
                acceleration_mode: AccelerationMode::Cpu,
                ..Default::default()
            },
        )
        .unwrap();

        let model = &VoiceModel::sample().await.unwrap();
        syntesizer.load_voice_model(model).await.unwrap();

        let accent_phrases = syntesizer
            .create_accent_phrases("同じ、文章、です。完全に、同一です。", StyleId::new(1))
            .await
            .unwrap();
        assert_eq!(accent_phrases.len(), 5);

        // 入力テキストに「、」や「。」などの句読点が含まれていたときに
        // AccentPhraseModel の pause_mora に期待する値をテスト

        assert!(
            accent_phrases[0].pause_mora().is_some(),
            "accent_phrases[0].pause_mora() is None"
        );
        assert!(
            accent_phrases[1].pause_mora().is_some(),
            "accent_phrases[1].pause_mora() is None"
        );
        assert!(
            accent_phrases[2].pause_mora().is_some(),
            "accent_phrases[2].pause_mora() is None"
        );
        assert!(
            accent_phrases[3].pause_mora().is_some(),
            "accent_phrases[3].pause_mora() is None"
        );
        assert!(
            accent_phrases[4].pause_mora().is_none(), // 文末の句読点は削除される
            "accent_phrases[4].pause_mora() is not None"
        );

        for accent_phrase in accent_phrases.iter().take(4) {
            let pause_mora = accent_phrase.pause_mora().clone().unwrap();
            assert_eq!(pause_mora.text(), "、");
            assert_eq!(pause_mora.consonant(), &None);
            assert_eq!(pause_mora.consonant_length(), &None);
            assert_eq!(pause_mora.vowel(), "pau");
            assert_ne!(
                pause_mora.vowel_length(),
                &0.0,
                "pause_mora.vowel_length() should not be 0.0"
            );
        }
    }

    #[rstest]
    #[tokio::test]
    async fn mora_length_works() {
        let syntesizer = Synthesizer::new(
            Arc::new(OpenJtalk::new(OPEN_JTALK_DIC_DIR).unwrap()),
            &InitializeOptions {
                acceleration_mode: AccelerationMode::Cpu,
                ..Default::default()
            },
        )
        .unwrap();

        let model = &VoiceModel::sample().await.unwrap();
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
                MoraModel::vowel_length
            ),
            "mora_length() does not work: mora.vowel_length() is not changed."
        );
    }

    #[rstest]
    #[tokio::test]
    async fn mora_pitch_works() {
        let syntesizer = Synthesizer::new(
            Arc::new(OpenJtalk::new(OPEN_JTALK_DIC_DIR).unwrap()),
            &InitializeOptions {
                acceleration_mode: AccelerationMode::Cpu,
                ..Default::default()
            },
        )
        .unwrap();

        let model = &VoiceModel::sample().await.unwrap();
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
            any_mora_param_changed(&accent_phrases, &modified_accent_phrases, MoraModel::pitch),
            "mora_pitch() does not work: mora.pitch() is not changed."
        );
    }

    #[rstest]
    #[tokio::test]
    async fn mora_data_works() {
        let syntesizer = Synthesizer::new(
            Arc::new(OpenJtalk::new(OPEN_JTALK_DIC_DIR).unwrap()),
            &InitializeOptions {
                acceleration_mode: AccelerationMode::Cpu,
                ..Default::default()
            },
        )
        .unwrap();

        let model = &VoiceModel::sample().await.unwrap();
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
            any_mora_param_changed(&accent_phrases, &modified_accent_phrases, MoraModel::pitch),
            "mora_data() does not work: mora.pitch() is not changed."
        );
        // NOTE: 一つでも母音の長さが変わっていれば、動作しているとみなす
        assert!(
            any_mora_param_changed(
                &accent_phrases,
                &modified_accent_phrases,
                MoraModel::vowel_length
            ),
            "mora_data() does not work: mora.vowel_length() is not changed."
        );
    }

    fn any_mora_param_changed<T: PartialEq>(
        before: &[AccentPhraseModel],
        after: &[AccentPhraseModel],
        param: fn(&MoraModel) -> &T,
    ) -> bool {
        std::iter::zip(before, after)
            .flat_map(move |(before, after)| std::iter::zip(before.moras(), after.moras()))
            .any(|(before, after)| param(before) != param(after))
    }

    enum Input {
        Japanese(&'static str),
        Kana(&'static str),
    }
}
