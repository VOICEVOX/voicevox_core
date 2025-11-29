mod frame_audio_query;
mod interpret_score;

pub(crate) use self::frame_audio_query::{KeyAndLyric, ValidatedNote};

pub use self::frame_audio_query::{FrameAudioQuery, FramePhoneme, Note, NoteId, Score};
