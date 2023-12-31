use world::{
    signal_analyzer::{AnalyzeResult, SignalAnalyzerBuilder},
    spectrogram_like::SpectrogramLike,
};

use crate::{
    error::ErrorRepr, synthesizer::DEFAULT_SAMPLING_RATE, AudioQueryModel, SpeakerMeta, StyleId,
};

use self::permit::MorphableTargets;

impl<O> crate::blocking::Synthesizer<O> {
    pub(crate) fn is_synthesis_morphing_permitted(
        &self,
        style_ids: MorphingPair<StyleId>,
        metas: &[SpeakerMeta],
    ) -> crate::Result<bool> {
        let pair = style_ids.lookup_speakers(metas)?;
        Ok(MorphableTargets::permit(pair).is_ok())
    }

    pub(crate) fn synthesis_morphing_(
        &self,
        audio_query: &AudioQueryModel,
        style_ids: MorphingPair<StyleId>,
        morph_rate: f32,
    ) -> crate::Result<Vec<u8>> {
        let metas = &self.metas();
        let pair = style_ids.lookup_speakers(metas)?;

        MorphableTargets::permit(pair)?.synthesis_morphing(self, audio_query, morph_rate)
    }
}

impl<'metas> MorphableTargets<'metas> {
    fn synthesis_morphing(
        self,
        synthesizer: &crate::blocking::Synthesizer<impl Sized>,
        audio_query: &AudioQueryModel,
        morph_rate: f32,
    ) -> crate::Result<Vec<u8>> {
        let morph_rate = f64::from(morph_rate);

        if *audio_query.output_sampling_rate() != DEFAULT_SAMPLING_RATE
            || *audio_query.output_stereo()
        {
            todo!();
        }

        let waves = &self.get().try_map(|style_id| {
            synthesizer.synthesis_wave(audio_query, style_id, &Default::default())
        })?;

        let morph_param = MorphingParameter::new(waves);

        let mut morph_spectrogram = SpectrogramLike::<f64>::new(
            morph_param.base_spectrogram.time_axis_size(),
            morph_param.base_spectrogram.frequency_axis_size(),
        );

        // FIXME: サイズ違いの場合は"resize"する
        for (morph_spectrogram, (base_spectrogram, target_spectrogram)) in itertools::zip_eq(
            morph_spectrogram.lines_mut(),
            itertools::zip_eq(
                morph_param.base_spectrogram.lines(),
                morph_param.target_spectrogram.lines(),
            ),
        ) {
            for (morph_spectrogram, (base_spectrogram, target_spectrogram)) in itertools::zip_eq(
                morph_spectrogram,
                itertools::zip_eq(base_spectrogram, target_spectrogram),
            ) {
                *morph_spectrogram = base_spectrogram * (1. - morph_rate) + target_spectrogram;
            }
        }

        let wave = &world::synthesis::synthesis(
            &morph_param.base_f0,
            &morph_spectrogram,
            &morph_param.base_aperiodicity,
            None,
            FRAME_PERIOD,
            DEFAULT_SAMPLING_RATE,
        )
        .unwrap_or_else(|_| {
            // FIXME: ここをどうするか考える。ただしここのエラーはspectrogramが巨大すぎる
            // (`world::synthesis::SynthesisError::TooLargeValue`)ときに限るはず
            todo!()
        });

        return Ok(super::to_wav(wave, audio_query));

        const FRAME_PERIOD: f64 = 1.;

        struct MorphingParameter {
            base_f0: Box<[f64]>,
            base_aperiodicity: SpectrogramLike<f64>,
            base_spectrogram: SpectrogramLike<f64>,
            target_spectrogram: SpectrogramLike<f64>,
        }

        impl MorphingParameter {
            fn new(wave: &MorphingPair<Vec<f32>>) -> Self {
                let (base_f0, base_spectrogram, base_aperiodicity) = analyze(&wave.base);
                let (_, target_spectrogram, _) = analyze(&wave.target);

                Self {
                    base_f0,
                    base_aperiodicity,
                    base_spectrogram,
                    target_spectrogram,
                }
            }
        }

        fn analyze(wave: &[f32]) -> (Box<[f64]>, SpectrogramLike<f64>, SpectrogramLike<f64>) {
            let analyzer = {
                let mut analyzer = SignalAnalyzerBuilder::new(DEFAULT_SAMPLING_RATE);
                analyzer.harvest_option_mut().set_frame_period(FRAME_PERIOD);
                analyzer.build(wave.iter().copied().map(Into::into).collect())
            };

            analyzer.calc_all();

            let AnalyzeResult {
                f0,
                spectrogram,
                aperiodicity,
                ..
            } = analyzer.into_result();

            let f0 = f0.expect("should be present");
            let spectrogram = spectrogram.expect("should be present");
            let aperiodicity = aperiodicity.expect("should be present");

            (f0, spectrogram, aperiodicity)
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct MorphingPair<T> {
    pub(crate) base: T,
    pub(crate) target: T,
}

impl<T> MorphingPair<T> {
    fn map<S>(self, mut f: impl FnMut(T) -> S) -> MorphingPair<S> {
        let base = f(self.base);
        let target = f(self.target);
        MorphingPair { base, target }
    }

    fn try_map<S, E>(
        self,
        mut f: impl FnMut(T) -> std::result::Result<S, E>,
    ) -> std::result::Result<MorphingPair<S>, E> {
        let base = f(self.base)?;
        let target = f(self.target)?;
        Ok(MorphingPair { base, target })
    }
}

impl MorphingPair<StyleId> {
    fn lookup_speakers(
        self,
        metas: &[SpeakerMeta],
    ) -> crate::Result<MorphingPair<(StyleId, &SpeakerMeta)>> {
        self.try_map(|style_id| {
            let speaker = metas
                .iter()
                .find(|m| m.styles().iter().any(|m| *m.id() == style_id))
                .ok_or(ErrorRepr::StyleNotFound { style_id })?;
            Ok((style_id, speaker))
        })
    }
}

mod permit {
    use std::marker::PhantomData;

    use crate::{
        error::{SpeakerFeatureError, SpeakerFeatureErrorKind},
        metas::PermittedSynthesisMorphing,
        SpeakerMeta, StyleId,
    };

    use super::MorphingPair;

    pub(super) struct MorphableTargets<'metas> {
        inner: MorphingPair<StyleId>,
        marker: PhantomData<&'metas ()>,
    }

    impl<'metas> MorphableTargets<'metas> {
        pub(super) fn permit(
            pair: MorphingPair<(StyleId, &'metas SpeakerMeta)>,
        ) -> std::result::Result<Self, SpeakerFeatureError> {
            match pair.map(|(_, speaker)| {
                (
                    speaker.supported_features().permitted_synthesis_morphing,
                    speaker,
                )
            }) {
                MorphingPair {
                    base: (PermittedSynthesisMorphing::All, _),
                    target: (PermittedSynthesisMorphing::All, _),
                } => {}

                MorphingPair {
                    base: (PermittedSynthesisMorphing::SelfOnly, base),
                    target: (PermittedSynthesisMorphing::SelfOnly, target),
                } if base.speaker_uuid() == target.speaker_uuid() => {}

                MorphingPair {
                    base: (_, base),
                    target: (_, target),
                } => {
                    return Err(SpeakerFeatureError {
                        speaker_name: base.name().clone(),
                        speaker_uuid: base.speaker_uuid().clone(),
                        context: SpeakerFeatureErrorKind::Morph {
                            target_speaker_name: target.name().clone(),
                            target_speaker_uuid: target.speaker_uuid().clone(),
                        },
                    })
                }
            }

            Ok(Self {
                inner: pair.map(|(style_id, _)| style_id),
                marker: PhantomData,
            })
        }

        pub(super) fn get(&self) -> MorphingPair<StyleId> {
            self.inner
        }
    }
}
