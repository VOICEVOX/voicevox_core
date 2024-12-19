use std::sync::LazyLock;

use regex::{Regex, Replacer};

use crate::assert_cdylib::Utf8Output;

macro_rules! static_regex {
    ($regex:expr $(,)?) => {{
        static REGEX: LazyLock<Regex> = LazyLock::new(|| $regex.parse().unwrap());
        &REGEX
    }};
}

impl Utf8Output {
    pub(crate) fn mask_timestamps(self) -> Self {
        self.mask_stderr(
            static_regex!(
                r"(?m)^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}.[0-9]{6}\+[0-9]{2}:[0-9]{2}",
            ),
            "{timestamp}",
        )
    }

    pub(crate) fn mask_onnxruntime_version(self) -> Self {
        self.mask_stderr(
            static_regex!(regex::escape(ort::downloaded_version!())),
            "{onnxruntime_version}",
        )
    }

    pub(crate) fn mask_windows_video_cards(self) -> Self {
        self.mask_stderr(
            static_regex!(
                r#"(?m)^\{timestamp\}  INFO voicevox_core::synthesizer: 検出されたGPU \(DirectMLにはGPU 0が使われます\):(\n\{timestamp\}  INFO voicevox_core::synthesizer:   GPU [0-9]+: "[^"]+" \([0-9.]+ [a-zA-Z]+\))+"#,
            ),
            "{windows-video-cards}",
        )
    }

    fn mask_stderr(self, regex: &Regex, rep: impl Replacer) -> Self {
        let stderr = regex.replace_all(&self.stderr, rep).into_owned();
        Self { stderr, ..self }
    }
}
