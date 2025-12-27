use std::num::NonZero;

use arrayvec::ArrayVec;
use tracing::warn;
use typeshare::U53;

use crate::{
    collections::{NonEmptyIterator, NonEmptyVec},
    error::{InvalidQueryError, InvalidQueryErrorSource},
};

use super::{
    super::{
        acoustic_feature_extractor::{NonPauBaseVowel, OptionalConsonant, PhonemeCode},
        sampling_rate::SamplingRate,
        validate::Validate as _,
    },
    queries::{FrameAudioQuery, FramePhoneme, Key, Note, NoteId, OptionalLyric, Score},
};

use self::note_seq::ValidatedNoteSeq;

/// 与えられた[楽譜]と[歌唱合成用のクエリ]の組み合わせが、基本周波数と音量の生成に利用できるかどうかを確認する。
///
/// # Errors
///
/// 次のうちどれかを満たすなら[`ErrorKind::InvalidQuery`]を表わすエラーを返す。
///
/// - `score`が[不正]。
/// - `score`と`frame_audio_query`が異なる音素列から成り立っている。ただし一部の音素は同一視される。
///
/// # Warnings
///
/// 次の状態に対しては[`WARN`]レベルのログを出す。将来的にはエラーになる予定。
///
/// - `frame_audio_query`が[警告を出す]状態。
///
/// [楽譜]: Score
/// [歌唱合成用のクエリ]: FrameAudioQuery
/// [`ErrorKind::InvalidQuery`]: crate::ErrorKind::InvalidQuery
/// [不正]: Score::validate
/// [`WARN`]: tracing::Level::WARN
/// [警告を出す]: FrameAudioQuery::validate
pub fn ensure_compatible(score: &Score, frame_audio_query: &FrameAudioQuery) -> crate::Result<()> {
    let ValidatedScore { notes } = score.try_into()?;
    frame_audio_query.validate();

    frame_phoneme_note_pairs(&frame_audio_query.phonemes, notes.as_ref())
        .map(|_| ())
        .map_err(|source| {
            InvalidQueryError {
                what: "`Score`と`FrameAudioQuery`の組み合わせ",
                value: None,
                source: Some(source),
            }
            .into()
        })
}

pub(crate) fn frame_phoneme_note_pairs<'a>(
    frame_phonemes: &'a [FramePhoneme],
    notes: &'a [ValidatedNote],
) -> Result<
    impl Iterator<Item = (&'a FramePhoneme, &'a ValidatedNote)> + Clone,
    InvalidQueryErrorSource,
> {
    let phonemes_from_query = frame_phonemes
        .iter()
        .map(|p| (PhonemeCode::from(p.phoneme.clone()), p));

    let phonemes_from_score = notes
        .iter()
        .flat_map(|note| note.phonemes().into_iter().map(move |p| (p, note)));

    if !itertools::equal(
        phonemes_from_query.clone().map(|(p, _)| p),
        phonemes_from_score.clone().map(|(p, _)| p),
    ) {
        return Err(InvalidQueryErrorSource::DifferentPhonemeSeqs);
    }

    Ok(itertools::zip_eq(
        phonemes_from_query.map(|(_, p)| p),
        phonemes_from_score.map(|(_, n)| n),
    ))
}

impl Score {
    /// この構造体が不正であるときエラーを返す。
    ///
    /// # Errors
    ///
    /// この構造体が不正であるとき[`ErrorKind::InvalidQuery`]を表わすエラーを返す。不正であるとは、以下の条件を満たすことである。
    ///
    /// - [`notes`]の要素のうちいずれかが[不正]。
    ///
    /// [`ErrorKind::InvalidQuery`]: crate::ErrorKind::InvalidQuery
    /// [`notes`]: Self::notes
    /// [不正]: Note::validate
    pub fn validate(&self) -> crate::Result<()> {
        ValidatedScore::try_from(self)
            .map(|_| ())
            .map_err(Into::into)
    }
}

impl Note {
    /// この構造体が不正であるときエラーを返す。
    ///
    /// # Errors
    ///
    /// この構造体が不正であるとき[`ErrorKind::InvalidQuery`]を表わすエラーを返す。不正であるとは、以下のいずれかの条件を満たすことである。
    ///
    /// - [`key`]が`None`かつ[`lyric`]が[`PAU`]以外。
    /// - [`key`]が`Some(_)`かつ[`lyric`]が[`PAU`]。
    ///
    /// [`ErrorKind::InvalidQuery`]: crate::ErrorKind::InvalidQuery
    /// [`key`]: Self::key
    /// [`lyric`]: Self::lyric
    /// [`PAU`]: OptionalLyric::PAU
    pub fn validate(&self) -> crate::Result<()> {
        ValidatedNote::try_from(self)
            .map(|_| ())
            .map_err(Into::into)
    }
}

impl FrameAudioQuery {
    pub(crate) fn total_frame_length(&self) -> usize {
        self.phonemes
            .iter()
            .map(|&FramePhoneme { frame_length, .. }| {
                typeshare::usize_from_u53_saturated(frame_length)
            })
            .sum()
    }

    /// 次の状態に対して[`WARN`]レベルのログを出す。
    ///
    /// - [`output_sampling_rate`]が`24000`以外の値（将来的に解消予定）。
    ///
    /// [`WARN`]: tracing::Level::WARN
    /// [`output_sampling_rate`]: Self::output_sampling_rate
    /// [#762]: https://github.com/VOICEVOX/voicevox_core/issues/762
    pub fn validate(&self) {
        if self.output_sampling_rate != SamplingRate::default() {
            warn!("`output_sampling_rate` should be `DEFAULT_SAMPLING_RATE`");
        }
    }

    pub(crate) fn warn_for_f0_len(&self) {
        let expected = self.total_frame_length();
        let actual = self.f0.len();
        if actual != expected {
            warn!(
                "length of `f0` should equal the total frame length: \
                 expected {expected}, got {actual}. the inference will fail"
            );
        }
    }

    pub(crate) fn warn_for_volume_len(&self) {
        let expected = self.total_frame_length();
        let actual = self.volume.len();
        if actual != expected {
            warn!(
                "length of `volume` should equal the total frame length: \
                 expected {expected}, got {actual}. the inference will fail"
            );
        }
    }
}

pub(crate) struct ValidatedScore {
    pub(crate) notes: ValidatedNoteSeq,
}

impl TryFrom<&'_ Score> for ValidatedScore {
    type Error = InvalidQueryError;

    fn try_from(score: &'_ Score) -> Result<Self, Self::Error> {
        let notes = (&*score.notes)
            .try_into()
            .map_err(|source| InvalidQueryError {
                what: Score::NAME,
                value: None,
                source: Some(InvalidQueryErrorSource::InvalidFields {
                    fields: "`notes`".to_owned(),
                    source: Box::new(source),
                }),
            })?;
        Ok(Self { notes })
    }
}

impl ValidatedNoteSeq {
    pub(crate) fn len(&self) -> NonZero<usize> {
        AsRef::<NonEmptyVec<_>>::as_ref(self).len()
    }

    pub(crate) fn iter(&self) -> impl NonEmptyIterator<Item = &ValidatedNote> {
        AsRef::<NonEmptyVec<_>>::as_ref(self).iter()
    }
}

impl<'a> TryFrom<&'a [Note]> for ValidatedNoteSeq {
    type Error = InvalidQueryError;

    fn try_from(notes: &'a [Note]) -> Result<Self, Self::Error> {
        let notes = notes
            .iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, _>>()?;

        NonEmptyVec::new(notes)
            .and_then(Into::into)
            .ok_or_else(|| InvalidQueryError {
                what: "ノート列",
                value: None,
                source: Some(InvalidQueryErrorSource::InitialNoteMustBePau),
            })
    }
}

impl AsRef<[ValidatedNote]> for ValidatedNoteSeq {
    fn as_ref(&self) -> &[ValidatedNote] {
        AsRef::<NonEmptyVec<_>>::as_ref(self).as_ref()
    }
}

pub(crate) struct ValidatedNote {
    pub(crate) id: Option<NoteId>,
    pub(crate) pau_or_key_and_lyric: PauOrKeyAndLyric,
    pub(crate) frame_length: U53,
}

impl ValidatedNote {
    fn phonemes(&self) -> ArrayVec<PhonemeCode, 2> {
        match self.pau_or_key_and_lyric {
            PauOrKeyAndLyric::Pau => [PhonemeCode::MorablePau].into_iter().collect(),
            // TODO: Rust 1.91以降なら`std::iter::chain`がある
            PauOrKeyAndLyric::KeyAndLyric {
                lyric:
                    Lyric {
                        phonemes: [(consonant, vowel)],
                        ..
                    },
                ..
            } => itertools::chain(Option::from(consonant), [vowel.into()]).collect(),
        }
    }
}

impl TryFrom<&'_ Note> for ValidatedNote {
    type Error = InvalidQueryError;

    fn try_from(note: &'_ Note) -> Result<Self, Self::Error> {
        let Note {
            id,
            key,
            lyric,
            frame_length,
        } = note;

        let pau_or_key_and_lyric = PauOrKeyAndLyric::new(*key, lyric)?;

        Ok(Self {
            id: id.clone(),
            pau_or_key_and_lyric,
            frame_length: *frame_length,
        })
    }
}

#[derive(PartialEq)]
pub(crate) enum PauOrKeyAndLyric {
    Pau,
    KeyAndLyric { key: Key, lyric: Lyric },
}

impl PauOrKeyAndLyric {
    fn new(key: Option<Key>, lyric: &OptionalLyric) -> Result<Self, InvalidQueryError> {
        match (key, &**lyric.phonemes()) {
            (None, []) => Ok(Self::Pau),
            (Some(key), &[mora]) => Ok(Self::KeyAndLyric {
                key,
                lyric: Lyric { phonemes: [mora] },
            }),
            (Some(_), []) => Err(InvalidQueryError {
                what: Note::NAME,
                value: None,
                source: Some(InvalidQueryErrorSource::UnnecessaryKeyForPau),
            }),
            (None, [_]) => Err(InvalidQueryError {
                what: Note::NAME,
                value: None,
                source: Some(InvalidQueryErrorSource::MissingKeyForNonPau),
            }),
            (_, [_, ..]) => unreachable!("the lyric should consist of at most one mora"),
        }
    }

    pub(crate) fn has_consonant(&self) -> bool {
        !matches!(
            self,
            Self::Pau
                | Self::KeyAndLyric {
                    lyric: Lyric {
                        phonemes: [(OptionalConsonant::None, _)],
                    },
                    ..
                }
        )
    }
}

#[derive(PartialEq)]
pub(crate) struct Lyric {
    pub(super) phonemes: [(OptionalConsonant, NonPauBaseVowel); 1],
}

pub(crate) mod note_seq {
    use derive_more::{AsRef, IntoIterator};

    use crate::collections::NonEmptyVec;

    use super::{PauOrKeyAndLyric, ValidatedNote};

    #[derive(AsRef, IntoIterator)]
    #[into_iterator(ref)]
    pub(crate) struct ValidatedNoteSeq(
        /// # Invariant
        ///
        /// The first note must be pau.
        NonEmptyVec<ValidatedNote>,
    );

    impl From<NonEmptyVec<ValidatedNote>> for Option<ValidatedNoteSeq> {
        fn from(notes: NonEmptyVec<ValidatedNote>) -> Self {
            (notes.first().pau_or_key_and_lyric == PauOrKeyAndLyric::Pau)
                .then_some(ValidatedNoteSeq(notes))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        error::{ErrorRepr, InvalidQueryError, InvalidQueryErrorSource},
        numerics::positive_finite_f32,
    };

    use super::super::{
        super::validate::Validate as _,
        queries::{FrameAudioQuery, FramePhoneme, Note, Score},
    };

    // TODO: トークの方と一緒に、`validated`に関するテストを書く

    #[test]
    fn ensure_compatible_works() {
        super::ensure_compatible(
            &score([
                note(None, ""),
                note(Some(0), "ド"),
                note(Some(0), "レ"),
                note(Some(0), "ミ"),
                note(None, ""),
            ]),
            &frame_audio_query([
                frame_phoneme("pau"),
                frame_phoneme("d"),
                frame_phoneme("o"),
                frame_phoneme("r"),
                frame_phoneme("e"),
                frame_phoneme("m"),
                frame_phoneme("i"),
                frame_phoneme("pau"),
            ]),
        )
        .unwrap();

        let err = super::ensure_compatible(
            &score([note(None, ""), note(Some(0), "ア")]),
            &frame_audio_query([frame_phoneme("pau"), frame_phoneme("i")]),
        )
        .unwrap_err();
        assert!(matches!(
            err,
            crate::Error(ErrorRepr::InvalidQuery(InvalidQueryError {
                what: "`Score`と`FrameAudioQuery`の組み合わせ",
                value: None,
                source: Some(InvalidQueryErrorSource::DifferentPhonemeSeqs),
            }))
        ));

        let err = super::ensure_compatible(
            &score([note(Some(0), "")]),
            &frame_audio_query([frame_phoneme("pau")]),
        )
        .unwrap_err();
        assert!(matches!(
            err,
            crate::Error(ErrorRepr::InvalidQuery(InvalidQueryError {
                what: Score::NAME,
                value: None,
                source: Some(InvalidQueryErrorSource::InvalidFields { .. }),
            }))
        ));

        fn score<const N: usize>(notes: [Note; N]) -> Score {
            Score {
                notes: notes.into(),
            }
        }

        fn note(key: Option<u8>, lyric: &str) -> Note {
            Note {
                id: None,
                key: key.map(|key| key.try_into().unwrap()),
                lyric: lyric.parse().unwrap(),
                frame_length: 1u8.into(),
            }
        }

        fn frame_audio_query<const N: usize>(phonemes: [FramePhoneme; N]) -> FrameAudioQuery {
            FrameAudioQuery {
                f0: [].into(),
                volume: [].into(),
                phonemes: phonemes.into(),
                volume_scale: positive_finite_f32!(1.),
                output_sampling_rate: Default::default(),
                output_stereo: true,
            }
        }

        fn frame_phoneme(phoneme: &str) -> FramePhoneme {
            FramePhoneme {
                phoneme: phoneme.parse().unwrap(),
                frame_length: 1u8.into(),
                note_id: None,
            }
        }
    }
}
