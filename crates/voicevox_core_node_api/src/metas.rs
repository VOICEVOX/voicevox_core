use napi::{Error, Result};
use voicevox_core::SpeakerMeta;

/// **話者**(_speaker_)のメタ情報。
#[napi(js_name = "SpeakerMeta")]
pub struct JsSpeakerMeta {
    handle: SpeakerMeta,
}

#[napi]
impl JsSpeakerMeta {
    /// @deprecated SpeakerMeta はコンストラクタによってコンストラクトできません。
    #[napi(constructor)]
    pub fn new() -> Result<JsSpeakerMeta> {
        Err(Error::from_reason(
            "SpeakerMeta はコンストラクタによってコンストラクトできません",
        ))
    }

    /// 話者名。
    #[napi(getter)]
    pub fn name(&self) -> String {
        self.handle.name().to_owned()
    }

    /// 話者に属するスタイル。
    #[napi(getter)]
    pub fn styles(&self) -> Vec<StyleMeta> {
        self.handle
            .styles()
            .iter()
            .map(|style| StyleMeta {
                id: style.id().to_string(),
                name: style.name().to_owned(),
                order: style.order().to_owned(),
            })
            .collect()
    }

    /// 話者のバージョン。
    #[napi(getter)]
    pub fn version(&self) -> String {
        self.handle.version().to_string()
    }

    /// 話者のUUID。
    #[napi(getter)]
    pub fn speaker_uuid(&self) -> String {
        self.handle.speaker_uuid().to_owned()
    }

    /// 話者の順番。
    ///
    /// `SpeakerMeta`の列は、この値に対して昇順に並んでいるべきである。
    #[napi(getter)]
    pub fn order(&self) -> Option<u32> {
        self.handle.order().to_owned()
    }
}

impl From<SpeakerMeta> for JsSpeakerMeta {
    fn from(value: SpeakerMeta) -> Self {
        JsSpeakerMeta { handle: value }
    }
}

/// **スタイル**(_style_)のメタ情報。
#[napi(object)]
pub struct StyleMeta {
    /// スタイルID。
    pub id: String,

    /// スタイル名。
    pub name: String,

    /// スタイルの順番。
    ///
    /// {@link SpeakerMeta.styles}は、この値に対して昇順に並んでいるべきである。
    pub order: Option<u32>,
}
