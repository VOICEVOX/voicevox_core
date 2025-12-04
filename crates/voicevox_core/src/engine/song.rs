mod frame_audio_query;
mod interpret;
mod validate;

pub(crate) use self::{
    interpret::{
        phoneme_lengths, repeat_phoneme_code_and_key, ConsonantLengthsFeature, PhonemeFeature,
        SfDecoderFeature,
    },
    validate::{join_frame_phonemes_with_notes, ValidatedNote, ValidatedNoteSeq, ValidatedScore},
};

pub use self::frame_audio_query::{
    FrameAudioQuery, FramePhoneme, Note, NoteId, OptionalLyric, Score,
};
