//! PCおよびGHAで動かすベンチマーク。GHAでの結果はCodSpeedにアップロードされる。
//!
//! [`CONFIG`]の内容は環境変数`VOICEVOX_CORE_BENCH_CONFIG`から上書きすることができる。
//!
//! # 例
//!
//! ```bash
//! #!/bin/bash
//! dir=$(realpath "$(dirname "$0")")
//! LD_LIBRARY_PATH="$dir/voicevox_core/onnxruntime/lib:$dir/voicevox_core/additional_libraries:$LD_LIBRARY_PATH" \
//!   VOICEVOX_CORE_BENCH_CONFIG='
//!     {
//!       "onnxruntime-path": null,
//!       "acceleration-mode":"GPU",
//!       "vvm": "../../voicevox_core/models/vvms/0.vvm",
//!       "include-long-input-text": true,
//!       "include-open-and-close-vvm": true,
//!       "iterate-more": true
//!     }' \
//!   cargo bench -p voicevox_core --features load-onnxruntime --bench benches
//! ```

use std::{env, path::PathBuf, sync::LazyLock};

use serde::Deserialize;
use voicevox_core::{AccelerationMode, StyleId};

#[derive(Clone, Copy, derive_more::Display)]
#[display("{name}")]
struct InputText {
    // `value`をテストケース名にしまうと視認性が著しく損なわれるため、短い識別子として`name`を用意
    name: &'static str,
    value: &'static str,
}

impl InputText {
    const SHORT: Self = Self {
        name: "SHORT",
        value: "この音声は、ボイスボックスを使用して、出力されています。",
    };

    const LONG: Self = Self {
        name: "LONG",
        value: "文章を書くとき一つの文にあれもこれも詰め込みたくなるが、\
            一般的に文章には適切な区切りが必要であり、\
            このような句読点がいくつもあるような文章は、\
            小説などでは例外かもしれないが少なくとも何かを説明するときには、\
            普通悪文とされる。",
    };
}

static CONFIG: LazyLock<Config> = {
    const ENV: &str = "VOICEVOX_CORE_BENCH_CONFIG";

    LazyLock::new(|| match env::var(ENV) {
        Ok(s) => serde_json::from_str(&s).unwrap(),
        Err(env::VarError::NotPresent) => serde_json::from_str("{}").expect("should not fail"),
        Err(env::VarError::NotUnicode(_)) => panic!("`${ENV}` is not valid UTF-8"),
    })
};

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
struct Config {
    /// (VOICEVOX) ONNX Runtimeのパス。`null`だとVOICEVOX ONNX Runtimeをシステムから探す。
    #[serde(default = "default_onnxruntime_path")]
    onnxruntime_path: Option<PathBuf>,

    /// アクセラレーションモード。
    #[serde(default)]
    acceleration_mode: CpuOrGpu,

    /// VVMのパス。
    #[serde(default = "default_vvm_path")]
    vvm: PathBuf,

    /// スタイルID。
    #[serde(default = "default_style_id")]
    style_id: StyleId,

    /// 長文の入力でのベンチマークも行う。
    #[serde(default)]
    include_long_input_text: bool,

    /// `open_and_close_vvm`を実行対象に含める。
    #[serde(default)]
    include_open_and_close_vvm: bool,

    /// イテレート回数を、ウォームアップも含めて増やす。
    #[serde(default)]
    iterate_more: bool,
}

fn default_onnxruntime_path() -> Option<PathBuf> {
    Some(test_util::ONNXRUNTIME_DYLIB_PATH.into())
}

fn default_vvm_path() -> PathBuf {
    test_util::SAMPLE_VOICE_MODEL_FILE_PATH.into()
}

/// 製品版VVMでは"四国めたん（あまあま）"。
fn default_style_id() -> StyleId {
    StyleId(0)
}

impl Config {
    fn input_text(&self) -> &[InputText] {
        if self.include_long_input_text {
            &[InputText::SHORT, InputText::LONG]
        } else {
            &[InputText::SHORT]
        }
    }

    fn iterations_for_light_operations(&self) -> Iterations {
        if self.iterate_more {
            Iterations {
                sample_count: 500,
                sample_size: 100,
                warmups: 50000,
            }
        } else {
            Iterations {
                sample_count: 100,
                sample_size: 50,
                warmups: 5000,
            }
        }
    }

    fn iterations_for_unload_and_load_vvm(&self) -> Iterations {
        if self.iterate_more {
            Iterations {
                sample_count: 50,
                sample_size: 2,
                warmups: 100,
            }
        } else {
            Iterations {
                sample_count: 10,
                sample_size: 1,
                warmups: 2,
            }
        }
    }

    fn iterations_for_synthesis(&self) -> Iterations {
        if self.iterate_more {
            Iterations {
                sample_count: 100,
                sample_size: 5,
                warmups: 1000,
            }
        } else {
            Iterations {
                sample_count: 10,
                sample_size: 1,
                warmups: 2,
            }
        }
    }
}

#[derive(Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
enum CpuOrGpu {
    #[default]
    Cpu,
    Gpu,
}

impl From<CpuOrGpu> for AccelerationMode {
    fn from(cpu_or_gpu: CpuOrGpu) -> Self {
        match cpu_or_gpu {
            CpuOrGpu::Cpu => Self::Cpu,
            CpuOrGpu::Gpu => Self::Gpu,
        }
    }
}

#[derive(Clone, Copy)]
struct Iterations {
    sample_count: u32,
    sample_size: u32,
    warmups: u32,
}

fn main() {
    let mut ort = voicevox_core::blocking::Onnxruntime::load_once();
    if let Some(onnxruntime_path) = &CONFIG.onnxruntime_path {
        ort = ort.filename(onnxruntime_path);
    }
    ort.perform().unwrap();

    let _ = *blocking::FIXTURE;
    let _ = *nonblocking::FIXTURE;

    divan::main();
}

mod blocking {
    use std::sync::LazyLock;

    use divan::Bencher;
    use voicevox_core::blocking::{
        Onnxruntime, OpenJtalk, Synthesizer, TextAnalyzer as _, VoiceModelFile,
    };

    use crate::{InputText, CONFIG};

    pub(crate) static FIXTURE: LazyLock<(Synthesizer<OpenJtalk>, VoiceModelFile)> =
        LazyLock::new(|| {
            let ort = Onnxruntime::get().expect("should have been initialized");
            let ojt = OpenJtalk::new(test_util::OPEN_JTALK_DIC_DIR).unwrap();
            let synth = Synthesizer::builder(ort)
                .text_analyzer(ojt)
                .acceleration_mode(CONFIG.acceleration_mode.into())
                .build()
                .unwrap();
            let vvm = VoiceModelFile::open(&CONFIG.vvm).unwrap();
            synth.load_voice_model(&vvm).unwrap();
            (synth, vvm)
        });

    #[divan::bench(
        sample_count = CONFIG.iterations_for_light_operations().sample_count,
        sample_size = CONFIG.iterations_for_light_operations().sample_size,
    )]
    fn construct_open_jtalk(bencher: Bencher<'_, '_>) {
        let run = || {
            divan::black_box_drop(OpenJtalk::new(test_util::OPEN_JTALK_DIC_DIR).unwrap());
        };
        for _ in 0..CONFIG.iterations_for_light_operations().warmups {
            run();
        }
        bencher.bench_local(run);
    }

    #[divan::bench(
        args = CONFIG.input_text(),
        sample_count = CONFIG.iterations_for_light_operations().sample_count,
        sample_size = CONFIG.iterations_for_light_operations().sample_size,
    )]
    fn analyze_text(bencher: Bencher<'_, '_>, input: InputText) {
        let ojt = FIXTURE.0.text_analyzer();
        let run = || ojt.analyze(input.value).unwrap();
        for _ in 0..CONFIG.iterations_for_light_operations().warmups {
            run();
        }
        bencher.bench_local(run);
    }

    #[divan::bench(
        sample_count = CONFIG.iterations_for_light_operations().sample_count,
        sample_size = CONFIG.iterations_for_light_operations().sample_size,
        ignore = !CONFIG.include_open_and_close_vvm,
    )]
    fn open_and_close_vvm(bencher: Bencher<'_, '_>) {
        let run = || {
            divan::black_box_drop(VoiceModelFile::open(&CONFIG.vvm).unwrap());
        };
        for _ in 0..CONFIG.iterations_for_light_operations().warmups {
            run();
        }
        bencher.bench_local(run);
    }

    #[divan::bench(
        sample_count = CONFIG.iterations_for_unload_and_load_vvm().sample_count,
        sample_size = CONFIG.iterations_for_unload_and_load_vvm().sample_size,
    )]
    fn unload_and_load_vvm(bencher: Bencher<'_, '_>) {
        let (synth, vvm) = &*FIXTURE;

        let run = || {
            synth.unload_voice_model(vvm.id()).unwrap();
            synth.load_voice_model(vvm).unwrap();
        };
        for _ in 0..CONFIG.iterations_for_unload_and_load_vvm().warmups {
            run();
        }
        bencher.bench_local(run);
    }

    #[divan::bench(
        args = CONFIG.input_text(),
        sample_count = CONFIG.iterations_for_light_operations().sample_count,
        sample_size = CONFIG.iterations_for_light_operations().sample_size,
    )]
    fn replace_mora_pitch(bencher: Bencher<'_, '_>, input: InputText) {
        let (synth, _) = &*FIXTURE;

        let input = &synth.text_analyzer().analyze(input.value).unwrap();

        let run = || synth.replace_mora_pitch(input, CONFIG.style_id).unwrap();
        for _ in 0..CONFIG.iterations_for_light_operations().warmups {
            divan::black_box(run());
        }
        bencher.bench_local(run);
    }

    #[divan::bench(
        args = CONFIG.input_text(),
        sample_count = CONFIG.iterations_for_light_operations().sample_count,
        sample_size = CONFIG.iterations_for_light_operations().sample_size,
    )]
    fn replace_phoneme_length(bencher: Bencher<'_, '_>, input: InputText) {
        let (synth, _) = &*FIXTURE;

        let input = &synth.text_analyzer().analyze(input.value).unwrap();

        let run = || {
            synth
                .replace_phoneme_length(input, CONFIG.style_id)
                .unwrap()
        };
        for _ in 0..CONFIG.iterations_for_light_operations().warmups {
            divan::black_box(run());
        }
        bencher.bench_local(run);
    }

    #[divan::bench(
        args = CONFIG.input_text(),
        sample_count = CONFIG.iterations_for_synthesis().sample_count,
        sample_size = CONFIG.iterations_for_synthesis().sample_size,
    )]
    fn synthesis(bencher: Bencher<'_, '_>, input: InputText) {
        let (synth, _) = &*FIXTURE;

        let query = &synth
            .create_audio_query(input.value, CONFIG.style_id)
            .unwrap();

        let run = || synth.synthesis(query, CONFIG.style_id).perform().unwrap();
        for _ in 0..CONFIG.iterations_for_synthesis().warmups {
            divan::black_box(run());
        }
        bencher.bench_local(run);
    }
}

mod nonblocking {
    use std::sync::LazyLock;

    use divan::Bencher;
    use voicevox_core::nonblocking::{
        Onnxruntime, OpenJtalk, Synthesizer, TextAnalyzer as _, VoiceModelFile,
    };

    use crate::{InputText, CONFIG};

    pub(crate) static FIXTURE: LazyLock<(Synthesizer<OpenJtalk>, VoiceModelFile)> =
        LazyLock::new(|| {
            pollster::block_on(async {
                let ort = Onnxruntime::get().expect("should have been initialized");
                let ojt = OpenJtalk::new(test_util::OPEN_JTALK_DIC_DIR).await.unwrap();
                let synth = Synthesizer::builder(ort)
                    .text_analyzer(ojt)
                    .acceleration_mode(CONFIG.acceleration_mode.into())
                    .build()
                    .unwrap();
                let vvm = VoiceModelFile::open(&CONFIG.vvm).await.unwrap();
                synth.load_voice_model(&vvm).await.unwrap();
                (synth, vvm)
            })
        });

    #[divan::bench(
        sample_count = CONFIG.iterations_for_light_operations().sample_count,
        sample_size = CONFIG.iterations_for_light_operations().sample_size,
    )]
    fn construct_open_jtalk(bencher: Bencher<'_, '_>) {
        let run = || {
            divan::black_box_drop(
                pollster::block_on(OpenJtalk::new(test_util::OPEN_JTALK_DIC_DIR)).unwrap(),
            );
        };
        for _ in 0..CONFIG.iterations_for_light_operations().warmups {
            run();
        }
        bencher.bench_local(run);
    }

    #[divan::bench(
        args = CONFIG.input_text(),
        sample_count = CONFIG.iterations_for_light_operations().sample_count,
        sample_size = CONFIG.iterations_for_light_operations().sample_size,
    )]
    #[pollster::main]
    async fn analyze_text(bencher: Bencher<'_, '_>, input: InputText) {
        let ojt = FIXTURE.0.text_analyzer();
        let run = || pollster::block_on(ojt.analyze(input.value)).unwrap();
        for _ in 0..CONFIG.iterations_for_light_operations().warmups {
            run();
        }
        bencher.bench_local(run);
    }

    #[divan::bench(
        sample_count = CONFIG.iterations_for_light_operations().sample_count,
        sample_size = CONFIG.iterations_for_light_operations().sample_size,
        ignore = !CONFIG.include_open_and_close_vvm,
    )]
    fn open_and_close_vvm(bencher: Bencher<'_, '_>) {
        let run = || {
            divan::black_box_drop(pollster::block_on(VoiceModelFile::open(&CONFIG.vvm)).unwrap());
        };
        for _ in 0..CONFIG.iterations_for_light_operations().warmups {
            run();
        }
        bencher.bench_local(run);
    }

    #[divan::bench(
        sample_count = CONFIG.iterations_for_unload_and_load_vvm().sample_count,
        sample_size = CONFIG.iterations_for_unload_and_load_vvm().sample_size,
    )]
    #[pollster::main]
    async fn unload_and_load_vvm(bencher: Bencher<'_, '_>) {
        let (synth, vvm) = &*FIXTURE;

        let run = || {
            synth.unload_voice_model(vvm.id()).unwrap();
            pollster::block_on(synth.load_voice_model(vvm)).unwrap();
        };
        for _ in 0..CONFIG.iterations_for_unload_and_load_vvm().warmups {
            run();
        }
        bencher.bench_local(run);
    }

    #[divan::bench(
        args = CONFIG.input_text(),
        sample_count = CONFIG.iterations_for_light_operations().sample_count,
        sample_size = CONFIG.iterations_for_light_operations().sample_size,
    )]
    #[pollster::main]
    async fn replace_mora_pitch(bencher: Bencher<'_, '_>, input: InputText) {
        let (synth, _) = &*FIXTURE;

        let input = &synth.text_analyzer().analyze(input.value).await.unwrap();

        let run = || pollster::block_on(synth.replace_mora_pitch(input, CONFIG.style_id)).unwrap();
        for _ in 0..CONFIG.iterations_for_light_operations().warmups {
            divan::black_box(run());
        }
        bencher.bench_local(run);
    }

    #[divan::bench(
        args = CONFIG.input_text(),
        sample_count = CONFIG.iterations_for_light_operations().sample_count,
        sample_size = CONFIG.iterations_for_light_operations().sample_size,
    )]
    #[pollster::main]
    async fn replace_phoneme_length(bencher: Bencher<'_, '_>, input: InputText) {
        let (synth, _) = &*FIXTURE;

        let input = &synth.text_analyzer().analyze(input.value).await.unwrap();

        let run =
            || pollster::block_on(synth.replace_phoneme_length(input, CONFIG.style_id)).unwrap();
        for _ in 0..CONFIG.iterations_for_light_operations().warmups {
            divan::black_box(run());
        }
        bencher.bench_local(run);
    }

    #[divan::bench(
        args = CONFIG.input_text(),
        sample_count = CONFIG.iterations_for_synthesis().sample_count,
        sample_size = CONFIG.iterations_for_synthesis().sample_size,
    )]
    #[pollster::main]
    async fn synthesis(bencher: Bencher<'_, '_>, input: InputText) {
        let (synth, _) = &*FIXTURE;

        let query = &synth
            .create_audio_query(input.value, CONFIG.style_id)
            .await
            .unwrap();

        let run = || pollster::block_on(synth.synthesis(query, CONFIG.style_id).perform()).unwrap();
        for _ in 0..CONFIG.iterations_for_synthesis().warmups {
            divan::black_box(run());
        }
        bencher.bench_local(run);
    }
}
