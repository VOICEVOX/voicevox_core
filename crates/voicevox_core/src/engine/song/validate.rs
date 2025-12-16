use std::num::NonZero;

use arrayvec::ArrayVec;
use tracing::warn;
use typeshare::U53;

use crate::{
    collections::{NonEmptyIterator, NonEmptyVec},
    error::{ErrorRepr, InvalidQueryError, InvalidQueryErrorSource},
};

use super::{
    super::{
        acoustic_feature_extractor::{MoraTail, OptionalConsonant, PhonemeCode},
        sampling_rate::SamplingRate,
    },
    queries::{FrameAudioQuery, FramePhoneme, Note, NoteId, OptionalLyric, Score},
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
    let ValidatedScore { notes } = score.to_validated()?;
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
    /// この構造体をバリデートする。
    ///
    /// # Errors
    ///
    /// 次を満たすなら[`ErrorKind::InvalidQuery`]を表わすエラーを返す。
    ///
    /// - [`notes`]の要素のうちいずれかが[不正]。
    ///
    /// [`ErrorKind::InvalidQuery`]: crate::ErrorKind::InvalidQuery
    /// [`notes`]: Self::notes
    /// [不正]: Note::validate
    pub fn validate(&self) -> crate::Result<()> {
        self.to_validated().map(|_| ())
    }

    pub(crate) fn to_validated(&self) -> crate::Result<ValidatedScore> {
        let notes = (&*self.notes).try_into()?;
        Ok(ValidatedScore { notes })
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
        self.clone().into_validated().map(|_| ())
    }

    fn into_validated(self) -> crate::Result<ValidatedNote> {
        let Self {
            id,
            key,
            lyric,
            frame_length,
        } = self;

        let pau_or_key_and_lyric = PauOrKeyAndLyric::new(key, &lyric)?;

        Ok(ValidatedNote {
            id,
            pau_or_key_and_lyric,
            frame_length,
        })
    }
}

impl FrameAudioQuery {
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
}

pub(crate) struct ValidatedScore {
    pub(crate) notes: ValidatedNoteSeq,
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

#[derive(PartialEq)]
pub(crate) enum PauOrKeyAndLyric {
    Pau,
    KeyAndLyric { key: U53, lyric: Lyric },
}

impl PauOrKeyAndLyric {
    fn new(key: Option<U53>, lyric: &OptionalLyric) -> crate::Result<Self> {
        match (key, &**lyric.phonemes()) {
            (None, []) => Ok(Self::Pau),
            (Some(key), &[mora]) => Ok(Self::KeyAndLyric {
                key,
                lyric: Lyric { phonemes: [mora] },
            }),
            (Some(_), []) => Err(ErrorRepr::InvalidQuery(InvalidQueryError {
                what: "ノート",
                value: None,
                source: Some(InvalidQueryErrorSource::UnnecessaryKeyForPau),
            })
            .into()),
            (None, [_]) => Err(ErrorRepr::InvalidQuery(InvalidQueryError {
                what: "ノート",
                value: None,
                source: Some(InvalidQueryErrorSource::MissingKeyForNonPau),
            })
            .into()),
            (_, [_, ..]) => unreachable!("the lyric should consist of at most one mora"),
        }
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
    type Error = crate::Error;

    fn try_from(notes: &'a [Note]) -> Result<Self, Self::Error> {
        let notes = notes
            .iter()
            .cloned()
            .map(Note::into_validated)
            .collect::<Result<Vec<_>, _>>()?;

        NonEmptyVec::new(notes).and_then(Self::new_).ok_or_else(|| {
            InvalidQueryError {
                what: "ノート列",
                value: None,
                source: Some(InvalidQueryErrorSource::InitialNoteMustBePau),
            }
            .into()
        })
    }
}

impl AsRef<[ValidatedNote]> for ValidatedNoteSeq {
    fn as_ref(&self) -> &[ValidatedNote] {
        AsRef::<NonEmptyVec<_>>::as_ref(self).as_ref()
    }
}

pub(crate) mod note_seq {
    use derive_more::AsRef;

    use crate::collections::NonEmptyVec;

    use super::{PauOrKeyAndLyric, ValidatedNote};

    #[derive(AsRef)]
    pub(crate) struct ValidatedNoteSeq(
        /// # Invariant
        ///
        /// The first note must be pau.
        NonEmptyVec<ValidatedNote>,
    );

    impl ValidatedNoteSeq {
        pub(super) fn new_(notes: NonEmptyVec<ValidatedNote>) -> Option<Self> {
            (notes.first().pau_or_key_and_lyric == PauOrKeyAndLyric::Pau).then_some(Self(notes))
        }
    }
}
