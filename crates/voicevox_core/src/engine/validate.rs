use duplicate::duplicate_item;
use serde::de::DeserializeOwned;

use super::{
    song::queries::{FrameAudioQuery, FramePhoneme, Note, Score},
    talk::{AccentPhrase, AudioQuery, Mora},
};

pub trait Validate: DeserializeOwned {
    const NAME: &str;
    fn validate(&self) -> crate::Result<()>;

    fn validation_error_description() -> String {
        format!("不正な{}です", Self::NAME)
    }
}

#[duplicate_item(
    T S validation;
    [ AudioQuery ] [ "AudioQuery" ] [ Self::validate ];
    [ AccentPhrase ] [ "アクセント句" ] [ Self::validate ];
    [ Mora ] [ "モーラ" ] [ Self::validate ];
    [ Vec<AccentPhrase> ] [ "アクセント句の列" ] [ |this: &Self| this.iter().try_for_each(AccentPhrase::validate) ];
    [ Note ] [ "ノート" ] [ Self::validate ];
    [ Score ] [ "楽譜" ] [ Self::validate ];
    [ FramePhoneme ] [ "FramePhoneme" ] [ |_| Ok(()) ];
    [ FrameAudioQuery ] [ "FrameAudioQuery" ] [ |this: &Self| { this.validate(); Ok(()) } ];
)]
impl Validate for T {
    const NAME: &str = S;

    fn validate(&self) -> crate::Result<()> {
        (validation)(self)
    }
}
