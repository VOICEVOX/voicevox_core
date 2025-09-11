use voicevox_core::StyleId;

const STYLE_ID: StyleId = StyleId(0); // 四国めたん（あまあま）
const VVM: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/benches/0.vvm");

const SHORT_INPUT: InputText = InputText {
    name: "SHORT",
    value: "こんにちは",
};
const MIDDLE_INPUT: InputText = InputText {
    name: "MIDDLE",
    value: "この音声は、ボイスボックスを使用して、出力されています。",
};
const LONG_INPUT: InputText = InputText {
    name: "LONG",
    value: "文章を書くとき一つの文にあれもこれも詰め込みたくなるが、\
        一般的に文章には適切な区切りが必要であり、\
        このような句読点がいくつもあるような文章は、\
        小説などでは例外かもしれないが少なくとも何かを説明するときには、\
        普通悪文とされる。",
};

#[derive(Clone, Copy, derive_more::Display)]
#[display("<{name}>")]
struct InputText {
    // `value`をテストケース名にしまうと視認性が著しく損なわれるため、短い識別子として`name`を用意
    name: &'static str,
    value: &'static str,
}

fn main() {
    voicevox_core::blocking::Onnxruntime::load_once()
        .perform()
        .unwrap();
    let _ = *self::blocking::FIXTURE;
    let _ = *self::nonblocking::FIXTURE;

    divan::main();
}

mod blocking {
    use std::sync::LazyLock;

    use divan::Bencher;
    use voicevox_core::{
        blocking::{Onnxruntime, OpenJtalk, Synthesizer, VoiceModelFile},
        AccelerationMode,
    };

    use crate::{InputText, LONG_INPUT, MIDDLE_INPUT, SHORT_INPUT, STYLE_ID, VVM};

    pub(crate) static FIXTURE: LazyLock<(Synthesizer<OpenJtalk>, VoiceModelFile)> =
        LazyLock::new(|| {
            let ort = Onnxruntime::get().expect("should have been initialized");
            let ojt = OpenJtalk::new(test_util::OPEN_JTALK_DIC_DIR).unwrap();
            let synth = Synthesizer::builder(ort)
                .text_analyzer(ojt)
                .acceleration_mode(AccelerationMode::Gpu)
                .build()
                .unwrap();
            let vvm = VoiceModelFile::open(VVM).unwrap();
            synth.load_voice_model(&vvm).unwrap();
            (synth, vvm)
        });

    #[divan::bench(sample_count = 20)]
    fn unload_and_load_vvm(bencher: Bencher<'_, '_>) {
        let (synth, vvm) = &*FIXTURE;

        let run = || {
            synth.unload_voice_model(vvm.id()).unwrap();
            synth.load_voice_model(vvm).unwrap();
        };
        // warmup
        for _ in 0..3 {
            run();
        }
        bencher.bench_local(run);
        synth.load_voice_model(vvm).unwrap();
    }

    #[divan::bench(
        args = [LONG_INPUT, MIDDLE_INPUT, SHORT_INPUT],
        sample_count = 50,
        sample_size = 3
    )]
    fn synthesis(bencher: Bencher<'_, '_>, input: InputText) {
        let (synth, _) = &*FIXTURE;

        let query = &synth.create_audio_query(input.value, STYLE_ID).unwrap();

        let run = || synth.synthesis(query, STYLE_ID).perform().unwrap();
        // warmup
        for _ in 0..5 {
            divan::black_box(run());
        }
        bencher.bench_local(run);
    }

    #[divan::bench(
        args = [LONG_INPUT, MIDDLE_INPUT, SHORT_INPUT],
        sample_count = 50,
        sample_size = 3
    )]
    fn tts(bencher: Bencher<'_, '_>, input: InputText) {
        let (synth, _) = &*FIXTURE;

        let run = || synth.tts(input.value, STYLE_ID).perform().unwrap();
        // warmup
        for _ in 0..5 {
            divan::black_box(run());
        }
        bencher.bench_local(run);
    }
}

mod nonblocking {
    use std::sync::LazyLock;

    use divan::Bencher;
    use voicevox_core::{
        nonblocking::{Onnxruntime, OpenJtalk, Synthesizer, VoiceModelFile},
        AccelerationMode,
    };

    use crate::{InputText, LONG_INPUT, MIDDLE_INPUT, SHORT_INPUT, STYLE_ID, VVM};

    pub(crate) static FIXTURE: LazyLock<(Synthesizer<OpenJtalk>, VoiceModelFile)> =
        LazyLock::new(|| {
            pollster::block_on(async {
                let ort = Onnxruntime::get().expect("should have been initialized");
                let ojt = OpenJtalk::new(test_util::OPEN_JTALK_DIC_DIR).await.unwrap();
                let synth = Synthesizer::builder(ort)
                    .text_analyzer(ojt)
                    .acceleration_mode(AccelerationMode::Gpu)
                    .build()
                    .unwrap();
                let vvm = VoiceModelFile::open(VVM).await.unwrap();
                synth.load_voice_model(&vvm).await.unwrap();
                (synth, vvm)
            })
        });

    #[divan::bench(sample_count = 20)]
    #[pollster::main]
    async fn unload_and_load_vvm(bencher: Bencher<'_, '_>) {
        let (synth, vvm) = &*FIXTURE;

        let run = || {
            synth.unload_voice_model(vvm.id()).unwrap();
            pollster::block_on(synth.load_voice_model(vvm)).unwrap();
        };
        // warmup
        for _ in 0..3 {
            run();
        }
        bencher.bench_local(run);
        synth.load_voice_model(vvm).await.unwrap();
    }

    #[divan::bench(
        args = [LONG_INPUT, MIDDLE_INPUT, SHORT_INPUT],
        sample_count = 50,
        sample_size = 3
    )]
    #[pollster::main]
    async fn synthesis(bencher: Bencher<'_, '_>, input: InputText) {
        let (synth, _) = &*FIXTURE;

        let query = &synth
            .create_audio_query(input.value, STYLE_ID)
            .await
            .unwrap();

        let run = || pollster::block_on(synth.synthesis(query, STYLE_ID).perform()).unwrap();
        // warmup
        for _ in 0..5 {
            divan::black_box(run());
        }
        bencher.bench_local(run);
    }

    #[divan::bench(
        args = [LONG_INPUT, MIDDLE_INPUT, SHORT_INPUT],
        sample_count = 50,
        sample_size = 3
    )]
    #[pollster::main]
    async fn tts(bencher: Bencher<'_, '_>, input: InputText) {
        let (synth, _) = &*FIXTURE;

        let run = || pollster::block_on(synth.tts(input.value, STYLE_ID).perform()).unwrap();
        // warmup
        for _ in 0..5 {
            divan::black_box(run());
        }
        bencher.bench_local(run);
    }
}
