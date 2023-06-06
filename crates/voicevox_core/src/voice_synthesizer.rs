use std::sync::Arc;

use const_default::ConstDefault;
use duplicate::duplicate_item;

use crate::engine::{create_kana, parse_kana, AccentPhraseModel, OpenJtalk, SynthesisEngine};

use super::*;

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

#[derive(ConstDefault)]
pub struct AccentPhrasesOptions {
    pub kana: bool,
}

#[derive(ConstDefault)]
pub struct AudioQueryOptions {
    pub kana: bool,
}

impl From<&TtsOptions> for AudioQueryOptions {
    fn from(options: &TtsOptions) -> Self {
        Self { kana: options.kana }
    }
}

pub struct TtsOptions {
    pub kana: bool,
    pub enable_interrogative_upspeak: bool,
}

impl AsRef<TtsOptions> for TtsOptions {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl ConstDefault for TtsOptions {
    const DEFAULT: Self = Self {
        enable_interrogative_upspeak: true,
        kana: ConstDefault::DEFAULT,
    };
}

#[derive(Debug, PartialEq, Eq)]
pub enum AccelerationMode {
    Auto,
    Cpu,
    Gpu,
}

impl ConstDefault for AccelerationMode {
    const DEFAULT: Self = Self::Auto;
}

#[derive(ConstDefault)]
pub struct InitializeOptions {
    pub acceleration_mode: AccelerationMode,
    pub cpu_num_threads: u16,
    pub load_all_models: bool,
}

#[duplicate_item(
    T;
    [ AccentPhrasesOptions ];
    [ AudioQueryOptions ];
    [ TtsOptions ];
    [ AccelerationMode ];
    [ InitializeOptions ];
)]
impl Default for T {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// 音声シンセサイザ
pub struct Synthesizer {
    synthesis_engine: SynthesisEngine,
    use_gpu: bool,
}

impl Synthesizer {
    /// コンストラクタ兼初期化
    pub async fn new_with_initialize(
        open_jtalk: Arc<OpenJtalk>,
        options: &InitializeOptions,
    ) -> Result<Self> {
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

        Ok(Self {
            synthesis_engine: SynthesisEngine::new(
                InferenceCore::new_with_initialize(
                    use_gpu,
                    options.cpu_num_threads,
                    options.load_all_models,
                )
                .await?,
                open_jtalk,
            ),
            use_gpu,
        })
    }

    pub fn is_gpu_mode(&self) -> bool {
        self.use_gpu
    }

    /// 音声モデルを読み込む
    pub async fn load_voice_model(&mut self, model: &VoiceModel) -> Result<()> {
        self.synthesis_engine
            .inference_core_mut()
            .load_model(model)
            .await?;
        Ok(())
    }

    /// 指定したモデルIdの音声モデルを開放する
    pub fn unload_voice_model(&mut self, voice_model_id: &VoiceModelId) -> Result<()> {
        self.synthesis_engine
            .inference_core_mut()
            .unload_model(voice_model_id)
    }

    /// 指定したモデルIdの音声モデルが読み込まれているか判定する
    pub fn is_loaded_voice_model(&self, voice_model_id: &VoiceModelId) -> bool {
        self.synthesis_engine
            .inference_core()
            .is_loaded_model(voice_model_id)
    }

    #[doc(hidden)]
    pub fn is_loaded_model_by_style_id(&self, style_id: StyleId) -> bool {
        self.synthesis_engine
            .inference_core()
            .is_model_loaded_by_style_id(style_id)
    }

    /// 今読み込んでいる音声モデルのメタ情報を返す
    pub fn metas(&self) -> &VoiceModelMeta {
        self.synthesis_engine.inference_core().metas()
    }

    /// 音声合成を行う
    pub async fn synthesis(
        &self,
        audio_query: &AudioQueryModel,
        style_id: StyleId,
        options: &SynthesisOptions,
    ) -> Result<Vec<u8>> {
        self.synthesis_engine
            .synthesis_wave_format(audio_query, style_id, options.enable_interrogative_upspeak)
            .await
    }

    #[doc(hidden)]
    pub async fn predict_duration(
        &self,
        phoneme_vector: &[i64],
        style_id: StyleId,
    ) -> Result<Vec<f32>> {
        self.synthesis_engine
            .inference_core()
            .predict_duration(phoneme_vector, style_id)
            .await
    }

    #[allow(clippy::too_many_arguments)]
    #[doc(hidden)]
    pub async fn predict_intonation(
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
        self.synthesis_engine
            .inference_core()
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
            .await
    }
    #[doc(hidden)]
    pub async fn decode(
        &self,
        length: usize,
        phoneme_size: usize,
        f0: &[f32],
        phoneme_vector: &[f32],
        style_id: StyleId,
    ) -> Result<Vec<f32>> {
        self.synthesis_engine
            .inference_core()
            .decode(length, phoneme_size, f0, phoneme_vector, style_id)
            .await
    }

    pub async fn create_accent_phrases(
        &self,
        text: &str,
        style_id: StyleId,
        options: &AccentPhrasesOptions,
    ) -> Result<Vec<AccentPhraseModel>> {
        if !self.synthesis_engine.is_openjtalk_dict_loaded() {
            return Err(Error::NotLoadedOpenjtalkDict);
        }
        if options.kana {
            self.synthesis_engine
                .replace_mora_data(&parse_kana(text)?, style_id)
                .await
        } else {
            self.synthesis_engine
                .create_accent_phrases(text, style_id)
                .await
        }
    }

    pub async fn replace_mora_data(
        &self,
        accent_phrases: &[AccentPhraseModel],
        style_id: StyleId,
    ) -> Result<Vec<AccentPhraseModel>> {
        self.synthesis_engine
            .replace_mora_data(accent_phrases, style_id)
            .await
    }

    pub async fn replace_phoneme_length(
        &self,
        accent_phrases: &[AccentPhraseModel],
        style_id: StyleId,
    ) -> Result<Vec<AccentPhraseModel>> {
        self.synthesis_engine
            .replace_phoneme_length(accent_phrases, style_id)
            .await
    }

    pub async fn replace_mora_pitch(
        &self,
        accent_phrases: &[AccentPhraseModel],
        style_id: StyleId,
    ) -> Result<Vec<AccentPhraseModel>> {
        self.synthesis_engine
            .replace_mora_pitch(accent_phrases, style_id)
            .await
    }

    pub async fn audio_query(
        &self,
        text: &str,
        style_id: StyleId,
        options: &AudioQueryOptions,
    ) -> Result<AudioQueryModel> {
        let accent_phrases = self
            .create_accent_phrases(text, style_id, &AccentPhrasesOptions { kana: options.kana })
            .await?;
        let kana = create_kana(&accent_phrases);
        Ok(AudioQueryModel::new(
            accent_phrases,
            1.,
            0.,
            1.,
            1.,
            0.1,
            0.1,
            SynthesisEngine::DEFAULT_SAMPLING_RATE,
            false,
            Some(kana),
        ))
    }

    pub async fn tts(
        &self,
        text: &str,
        style_id: StyleId,
        options: &TtsOptions,
    ) -> Result<Vec<u8>> {
        let audio_query = &self
            .audio_query(text, style_id, &AudioQueryOptions::from(options))
            .await?;
        self.synthesis(audio_query, style_id, &SynthesisOptions::from(options))
            .await
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

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{engine::MoraModel, macros::tests::assert_debug_fmt_eq};
    use ::test_util::OPEN_JTALK_DIC_DIR;

    #[rstest]
    #[case(Ok(()))]
    #[tokio::test]
    async fn load_model_works(#[case] expected_result_at_initialized: Result<()>) {
        let mut syntesizer = Synthesizer::new_with_initialize(
            Arc::new(OpenJtalk::new_without_dic()),
            &InitializeOptions {
                acceleration_mode: AccelerationMode::Cpu,
                ..Default::default()
            },
        )
        .await
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
        let syntesizer = Synthesizer::new_with_initialize(
            Arc::new(OpenJtalk::new_without_dic()),
            &InitializeOptions {
                acceleration_mode: AccelerationMode::Cpu,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert!(!syntesizer.is_gpu_mode());
    }

    #[rstest]
    #[case(1, true)]
    #[tokio::test]
    async fn is_loaded_model_by_style_id_works(#[case] style_id: u32, #[case] expected: bool) {
        let style_id = StyleId::new(style_id);
        let mut syntesizer = Synthesizer::new_with_initialize(
            Arc::new(OpenJtalk::new_without_dic()),
            &InitializeOptions {
                acceleration_mode: AccelerationMode::Cpu,
                ..Default::default()
            },
        )
        .await
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
        let mut syntesizer = Synthesizer::new_with_initialize(
            Arc::new(OpenJtalk::new_without_dic()),
            &InitializeOptions {
                acceleration_mode: AccelerationMode::Cpu,
                ..Default::default()
            },
        )
        .await
        .unwrap();

        syntesizer
            .load_voice_model(&open_default_vvm_file().await)
            .await
            .unwrap();

        // 「こんにちは、音声合成の世界へようこそ」という文章を変換して得た phoneme_vector
        let phoneme_vector = [
            0, 23, 30, 4, 28, 21, 10, 21, 42, 7, 0, 30, 4, 35, 14, 14, 16, 30, 30, 35, 14, 14, 28,
            30, 35, 14, 23, 7, 21, 14, 43, 30, 30, 23, 30, 35, 30, 0,
        ];

        let result = syntesizer
            .predict_duration(&phoneme_vector, StyleId::new(1))
            .await;

        assert!(result.is_ok(), "{result:?}");
        assert_eq!(result.unwrap().len(), phoneme_vector.len());
    }

    #[rstest]
    #[tokio::test]
    async fn predict_intonation_works() {
        let mut syntesizer = Synthesizer::new_with_initialize(
            Arc::new(OpenJtalk::new_without_dic()),
            &InitializeOptions {
                acceleration_mode: AccelerationMode::Cpu,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        syntesizer
            .load_voice_model(&open_default_vvm_file().await)
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
        let mut syntesizer = Synthesizer::new_with_initialize(
            Arc::new(OpenJtalk::new_without_dic()),
            &InitializeOptions {
                acceleration_mode: AccelerationMode::Cpu,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        syntesizer
            .load_voice_model(&open_default_vvm_file().await)
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
        "これはテストです",
        false,
        TEXT_CONSONANT_VOWEL_DATA1,
        "コレワ'/テ'_ストデ_ス"
    )]
    #[case(
        "コ'レワ/テ_スト'デ_ス",
        true,
        TEXT_CONSONANT_VOWEL_DATA2,
        "コ'レワ/テ_スト'デ_ス"
    )]
    #[tokio::test]
    async fn audio_query_works(
        #[case] input_text: &str,
        #[case] input_kana_option: bool,
        #[case] expected_text_consonant_vowel_data: &TextConsonantVowelData,
        #[case] expected_kana_text: &str,
    ) {
        let syntesizer = Synthesizer::new_with_initialize(
            Arc::new(OpenJtalk::new_with_initialize(OPEN_JTALK_DIC_DIR).unwrap()),
            &InitializeOptions {
                acceleration_mode: AccelerationMode::Cpu,
                load_all_models: true,
                ..Default::default()
            },
        )
        .await
        .unwrap();

        let query = syntesizer
            .audio_query(
                input_text,
                StyleId::new(0),
                &AudioQueryOptions {
                    kana: input_kana_option,
                },
            )
            .await
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
    #[case("これはテストです", false, TEXT_CONSONANT_VOWEL_DATA1)]
    #[case("コ'レワ/テ_スト'デ_ス", true, TEXT_CONSONANT_VOWEL_DATA2)]
    #[tokio::test]
    async fn accent_phrases_works(
        #[case] input_text: &str,
        #[case] input_kana_option: bool,
        #[case] expected_text_consonant_vowel_data: &TextConsonantVowelData,
    ) {
        let syntesizer = Synthesizer::new_with_initialize(
            Arc::new(OpenJtalk::new_with_initialize(OPEN_JTALK_DIC_DIR).unwrap()),
            &InitializeOptions {
                acceleration_mode: AccelerationMode::Cpu,
                load_all_models: true,
                ..Default::default()
            },
        )
        .await
        .unwrap();

        let accent_phrases = syntesizer
            .create_accent_phrases(
                input_text,
                StyleId::new(0),
                &AccentPhrasesOptions {
                    kana: input_kana_option,
                },
            )
            .await
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
    async fn mora_length_works() {
        let syntesizer = Synthesizer::new_with_initialize(
            Arc::new(OpenJtalk::new_with_initialize(OPEN_JTALK_DIC_DIR).unwrap()),
            &InitializeOptions {
                acceleration_mode: AccelerationMode::Cpu,
                load_all_models: true,
                ..Default::default()
            },
        )
        .await
        .unwrap();

        let accent_phrases = syntesizer
            .create_accent_phrases(
                "これはテストです",
                StyleId::new(0),
                &AccentPhrasesOptions { kana: false },
            )
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
        let syntesizer = Synthesizer::new_with_initialize(
            Arc::new(OpenJtalk::new_with_initialize(OPEN_JTALK_DIC_DIR).unwrap()),
            &InitializeOptions {
                acceleration_mode: AccelerationMode::Cpu,
                load_all_models: true,
                ..Default::default()
            },
        )
        .await
        .unwrap();

        let accent_phrases = syntesizer
            .create_accent_phrases(
                "これはテストです",
                StyleId::new(0),
                &AccentPhrasesOptions { kana: false },
            )
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
        let syntesizer = Synthesizer::new_with_initialize(
            Arc::new(OpenJtalk::new_with_initialize(OPEN_JTALK_DIC_DIR).unwrap()),
            &InitializeOptions {
                acceleration_mode: AccelerationMode::Cpu,
                load_all_models: true,
                ..Default::default()
            },
        )
        .await
        .unwrap();

        let accent_phrases = syntesizer
            .create_accent_phrases(
                "これはテストです",
                StyleId::new(0),
                &AccentPhrasesOptions { kana: false },
            )
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
}
