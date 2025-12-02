mod frame_audio_query;
mod interpret_query;
mod interpret_score;

pub(crate) use self::{
    frame_audio_query::{ValidatedNote, ValidatedNoteSeq, ValidatedNoteSeqWithConsonantLengths},
    interpret_query::SfDecoderFeature,
    interpret_score::{phoneme_lengths, ScoreFeature},
};

pub use self::frame_audio_query::{
    FrameAudioQuery, FramePhoneme, Note, NoteId, OptionalLyric, Score,
};
