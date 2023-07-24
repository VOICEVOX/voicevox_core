use std::{num::NonZeroU16, sync::Arc};

use criterion::{criterion_group, criterion_main, Criterion};
use test_util::OPEN_JTALK_DIC_DIR;
use tokio::{join, runtime::Runtime};
use voicevox_core::{
    AccelerationMode, AudioQueryModel, InitializeOptions, LoadVoiceModelOptions, OpenJtalk,
    StyleId, SynthesisOptions, Synthesizer, VoiceModel,
};

criterion_main!(benches);
criterion_group!(benches, benchmark);

fn benchmark(c: &mut Criterion) {
    let (synthesizer, aq) = &Runtime::new().unwrap().block_on(setup()).unwrap();

    let decode = || async {
        synthesizer
            .synthesis(
                aq,
                StyleId::new(0),
                &SynthesisOptions {
                    enable_interrogative_upspeak: true,
                },
            )
            .await
            .unwrap()
    };

    c.bench_function("decode_parallel", |b| {
        b.to_async(Runtime::new().unwrap())
            .iter(|| async { join!(decode(), decode(), decode(), decode()) })
    });

    c.bench_function("decode_sequential", |b| {
        b.to_async(Runtime::new().unwrap()).iter(|| async {
            for _ in 0..4 {
                decode().await;
            }
        })
    });
}

async fn setup() -> voicevox_core::Result<(Synthesizer, AudioQueryModel)> {
    let syntesizer = Synthesizer::new_with_initialize(
        Arc::new(OpenJtalk::new_with_initialize(OPEN_JTALK_DIC_DIR).unwrap()),
        &InitializeOptions {
            acceleration_mode: AccelerationMode::Gpu,
            cpu_num_threads: 4,
            ..Default::default()
        },
    )
    .await?;

    let model = &VoiceModel::from_path(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../model/sample.vvm",
    ))
    .await?;
    syntesizer
        .load_voice_model(
            model,
            &LoadVoiceModelOptions {
                gpu_num_sessions: NonZeroU16::new(4).unwrap(),
            },
        )
        .await?;

    let aq = syntesizer
        .audio_query(
            "寿限無寿限無五劫の擦り切れ海砂利水魚の水行末雲来末",
            StyleId::new(0),
            &Default::default(),
        )
        .await?;

    Ok((syntesizer, aq))
}
