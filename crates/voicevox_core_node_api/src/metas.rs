use napi::{Error, Result};
use voicevox_core::SpeakerMeta;

/// **話者**(_speaker_)のメタ情報。
#[napi(js_name = "SpeakerMeta")]
pub struct JsSpeakerMeta {
    speaker_meta: SpeakerMeta,
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
        self.speaker_meta.name().to_owned()
    }

    /// 話者に属するスタイル。
    #[napi(getter)]
    pub fn styles(&self) -> Vec<StyleMeta> {
        self.speaker_meta
            .styles()
            .iter()
            .map(|style| StyleMeta {
                id: style.id().raw_id(),
                name: style.name().to_owned(),
                order: style.order().to_owned(),
            })
            .collect()
    }

    /// 話者のバージョン。
    #[napi(getter)]
    pub fn version(&self) -> String {
        self.speaker_meta.version().to_string()
    }

    /// 話者のUUID。
    #[napi(getter)]
    pub fn speaker_uuid(&self) -> String {
        self.speaker_meta.speaker_uuid().to_owned()
    }

    /// 話者の順番。
    ///
    /// `SpeakerMeta`の列は、この値に対して昇順に並んでいるべきである。
    #[napi(getter)]
    pub fn order(&self) -> Option<u32> {
        self.speaker_meta.order().to_owned()
    }
}

impl From<SpeakerMeta> for JsSpeakerMeta {
    fn from(value: SpeakerMeta) -> Self {
        JsSpeakerMeta {
            speaker_meta: value,
        }
    }
}

/// **スタイル**(_style_)のメタ情報。
#[napi(object)]
pub struct StyleMeta {
    /// スタイルID。
    pub id: u32,

    /// スタイル名。
    pub name: String,

    /// スタイルの順番。
    ///
    /// {@link SpeakerMeta.styles}は、この値に対して昇順に並んでいるべきである。
    pub order: Option<u32>,
}
