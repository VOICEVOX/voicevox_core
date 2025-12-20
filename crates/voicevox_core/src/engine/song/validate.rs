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
        acoustic_feature_extractor::{MoraTail, OptionalConsonant, PhonemeCode},
        sampling_rate::SamplingRate,
    },
    queries::{FrameAudioQuery, FramePhoneme, Note, NoteId, OptionalLyric, Score},
};

use self::{
    validated_frame_audio_query::ValidatedFrameAudioQuery, validated_note_seq::ValidatedNoteSeq,
};

/// 与えられた[楽譜]と[歌唱合成用のクエリ]の組み合わせが、基本周波数と音量の生成に利用できるかどうかを確認する。
///
/// # Errors
///
/// 次のうちどれかを満たすなら[`ErrorKind::InvalidQuery`]を表わすエラーを返す。
///
/// - `score`が[不正](./struct.Score.html#method.validate)。
/// - `frame_audio_query`が[不正](./struct.FrameAudioQuery.html#method.validate)。
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
/// [`WARN`]: tracing::Level::WARN
/// [警告を出す]: FrameAudioQuery::validate
pub fn ensure_compatible(score: &Score, frame_audio_query: &FrameAudioQuery) -> crate::Result<()> {
    let ValidatedScore { notes } = score.try_into()?;
    let frame_audio_query = <&ValidatedFrameAudioQuery>::try_from(frame_audio_query)?;

    frame_phoneme_note_pairs(&frame_audio_query.as_ref().phonemes, notes.as_ref())
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
    /// この構造体をバリデートする。
    ///
    /// # Errors
    ///
    /// 次を満たすなら[`ErrorKind::InvalidQuery`]を表わすエラーを返す。
    ///
    /// - [`notes`]の要素のうちいずれかが[不正]。
    /// - [`Note::frame_length`]の合計が`0`。
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
    /// この構造体をバリデートする。
    ///
    /// # Errors
    ///
    /// 次のうちどれかを満たすなら[`ErrorKind::InvalidQuery`]を表わすエラーを返す。
    ///
    /// - [`key`]が`None`かつ[`lyric`]が[`PAU`]。
    /// - [`key`]が`Some(_)`かつ[`lyric`]が[`PAU`]以外。
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
    /// この構造体をバリデートする。
    ///
    /// # Errors
    ///
    /// 次の条件を満たすなら[`ErrorKind::InvalidQuery`]を表わすエラーを返す。
    ///
    /// - [`FramePhoneme::frame_length`]の合計が`0`。
    ///
    /// # Warnings
    ///
    /// 次の状態に対して[`WARN`]レベルのログを出す。
    ///
    /// - [`output_sampling_rate`]が`24000`以外の値（将来的に解消予定。cf. [#762]）。
    ///
    /// [`ErrorKind::InvalidQuery`]: crate::ErrorKind::InvalidQuery
    /// [`WARN`]: tracing::Level::WARN
    /// [`output_sampling_rate`]: Self::output_sampling_rate
    /// [#762]: https://github.com/VOICEVOX/voicevox_core/issues/762
    pub fn validate(&self) -> crate::Result<()> {
        if self.output_sampling_rate != SamplingRate::default() {
            warn!("`output_sampling_rate` should be `DEFAULT_SAMPLING_RATE`");
        }
        <&ValidatedFrameAudioQuery>::try_from(self)
            .map(|_| ())
            .map_err(Into::into)
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
                what: "楽譜",
                value: None,
                source: Some(InvalidQueryErrorSource::InvalidFields {
                    fields: "`notes`".to_owned(),
                    source: Box::new(source),
                }),
            })?;
        Ok(Self { notes })
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
            } => itertools::chain(consonant.try_into(), [vowel.into()]).collect(),
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
    KeyAndLyric { key: U53, lyric: Lyric },
}

impl PauOrKeyAndLyric {
    fn new(key: Option<U53>, lyric: &OptionalLyric) -> Result<Self, InvalidQueryError> {
        match (key, &**lyric.phonemes()) {
            (None, []) => Ok(Self::Pau),
            (Some(key), &[mora]) => Ok(Self::KeyAndLyric {
                key,
                lyric: Lyric { phonemes: [mora] },
            }),
            (Some(_), []) => Err(InvalidQueryError {
                what: "ノート",
                value: None,
                source: Some(InvalidQueryErrorSource::UnnecessaryKeyForPau),
            }),
            (None, [_]) => Err(InvalidQueryError {
                what: "ノート",
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
    // TODO: `NonPauBaseVowel`型 (= a | i | u | e | o | cl | N) を導入する
    pub(super) phonemes: [(OptionalConsonant, MoraTail); 1],
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
            .ok_or(InvalidQueryErrorSource::InitialNoteMustBePau)
            .and_then(TryInto::try_into)
            .map_err(|source| InvalidQueryError {
                what: "ノート列",
                value: None,
                source: Some(source),
            })
    }
}

impl AsRef<[ValidatedNote]> for ValidatedNoteSeq {
    fn as_ref(&self) -> &[ValidatedNote] {
        AsRef::<NonEmptyVec<_>>::as_ref(self).as_ref()
    }
}

impl ValidatedFrameAudioQuery {
    pub(crate) fn total_frame_length(&self) -> NonZero<usize> {
        self.as_ref()
            .phonemes
            .iter()
            .map(|&FramePhoneme { frame_length, .. }| {
                typeshare::usize_from_u53_saturated(frame_length)
            })
            .sum::<usize>()
            .try_into()
            .expect("the invariant should ensure")
    }

    pub(crate) fn warn_for_f0_len(&self) {
        let expected = self.total_frame_length().get();
        let actual = self.as_ref().f0.len();
        if actual != expected {
            warn!(
                "length of `f0` should equal the total frame length: \
                 expected {expected}, got {actual}. the inference will fail"
            );
        }
    }

    pub(crate) fn warn_for_volume_len(&self) {
        let expected = self.total_frame_length().get();
        let actual = self.as_ref().volume.len();
        if actual != expected {
            warn!(
                "length of `volume` should equal the total frame length: \
                 expected {expected}, got {actual}. the inference will fail"
            );
        }
    }
}

impl<'a> TryFrom<&'a FrameAudioQuery> for &'a ValidatedFrameAudioQuery {
    type Error = InvalidQueryError;

    fn try_from(frame_audio_query: &'a FrameAudioQuery) -> Result<Self, Self::Error> {
        Option::<_>::from(frame_audio_query).ok_or_else(|| InvalidQueryError {
            what: "FrameAudioQuery",
            value: None,
            source: Some(InvalidQueryErrorSource::TotalFrameLengthIsZero),
        })
    }
}

pub(crate) mod validated_note_seq {
    use derive_more::{AsRef, IntoIterator};
    use typeshare::U53;

    use crate::{
        collections::{NonEmptyIterator as _, NonEmptyVec},
        error::InvalidQueryErrorSource,
    };

    use super::{PauOrKeyAndLyric, ValidatedNote};

    #[derive(AsRef, IntoIterator)]
    #[into_iterator(ref)]
    pub(crate) struct ValidatedNoteSeq(
        /// # Invariant
        ///
        /// - The first note must be pau.
        /// - The sum of `frame_length`s must be non-zero.
        NonEmptyVec<ValidatedNote>,
    );

    impl TryFrom<NonEmptyVec<ValidatedNote>> for ValidatedNoteSeq {
        type Error = InvalidQueryErrorSource;

        fn try_from(notes: NonEmptyVec<ValidatedNote>) -> Result<Self, Self::Error> {
            if notes.first().pau_or_key_and_lyric != PauOrKeyAndLyric::Pau {
                return Err(InvalidQueryErrorSource::InitialNoteMustBePau);
            }
            if !notes
                .iter()
                .any(|&ValidatedNote { frame_length, .. }| frame_length > U53::from(0u8))
            {
                return Err(InvalidQueryErrorSource::TotalFrameLengthIsZero);
            }
            Ok(Self(notes))
        }
    }
}

pub(crate) mod validated_frame_audio_query {
    use derive_more::AsRef;
    use ref_cast::{ref_cast_custom, RefCastCustom};
    use typeshare::U53;

    use super::super::queries::{FrameAudioQuery, FramePhoneme};

    #[derive(AsRef, RefCastCustom)]
    #[repr(transparent)]
    pub(crate) struct ValidatedFrameAudioQuery(
        /// # Invariant
        ///
        /// The sum of `frame_length`s must be non-zero.
        FrameAudioQuery,
    );

    impl ValidatedFrameAudioQuery {
        #[ref_cast_custom]
        fn new(frame_audio_query: &FrameAudioQuery) -> &Self;
    }

    impl<'a> From<&'a FrameAudioQuery> for Option<&'a ValidatedFrameAudioQuery> {
        fn from(frame_audio_query: &'a FrameAudioQuery) -> Self {
            frame_audio_query
                .phonemes
                .iter()
                .any(|&FramePhoneme { frame_length, .. }| frame_length > U53::from(0u8))
                .then_some(ValidatedFrameAudioQuery::new(frame_audio_query))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::error::{ErrorRepr, InvalidQueryError, InvalidQueryErrorSource};

    use super::super::queries::FrameAudioQuery;
    use super::super::queries::{FramePhoneme, Note, Score};

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
                ..
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
                what: "楽譜",
                value: None,
                source: Some(InvalidQueryErrorSource::InvalidFields { .. }),
                ..
            }))
        ));

        let err =
            super::ensure_compatible(&score([note(None, "")]), &frame_audio_query([])).unwrap_err();
        assert!(matches!(
            err,
            crate::Error(ErrorRepr::InvalidQuery(InvalidQueryError {
                what: "FrameAudioQuery",
                value: None,
                source: Some(InvalidQueryErrorSource::TotalFrameLengthIsZero),
                ..
            }))
        ));

        fn score<const N: usize>(notes: [Note; N]) -> Score {
            Score {
                notes: notes.into(),
            }
        }

        fn note(key: Option<u32>, lyric: &str) -> Note {
            Note {
                id: None,
                key: key.map(Into::into),
                lyric: lyric.parse().unwrap(),
                frame_length: 1u8.into(),
            }
        }

        fn frame_audio_query<const N: usize>(phonemes: [FramePhoneme; N]) -> FrameAudioQuery {
            FrameAudioQuery {
                f0: [].into(),
                volume: [].into(),
                phonemes: phonemes.into(),
                volume_scale: (1.).try_into().unwrap(),
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
