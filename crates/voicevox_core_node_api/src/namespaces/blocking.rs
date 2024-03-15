/// ブロッキング版API。
#[napi]
pub mod blocking {
    use napi::{Error, Result};
    use uuid::Uuid;
    use voicevox_core::blocking::{OpenJtalk, Synthesizer, UserDict};

    use crate::convert_result;
    use crate::synthesizer::InitializeOptions;
    use crate::word::UserDictWord;

    /// テキスト解析器としてのOpen JTalk。
    #[napi(js_name = "OpenJtalk")]
    pub struct JsOpenJtalk {
        handle: OpenJtalk,
    }

    #[napi]
    impl JsOpenJtalk {
        #[napi(factory)]
        pub fn create(open_jtalk_dict_dir: String) -> Result<JsOpenJtalk> {
            Ok(JsOpenJtalk {
                handle: convert_result(OpenJtalk::new(open_jtalk_dict_dir))?,
            })
        }

        /// ユーザー辞書を設定する。
        ///
        /// この関数を呼び出した後にユーザー辞書を変更した場合は、再度この関数を呼ぶ必要がある。
        #[napi]
        pub fn use_user_dict(&self, user_dict: &JsUserDict) -> Result<()> {
            convert_result(self.handle.use_user_dict(&user_dict.handle))
        }
    }

    /// ユーザー辞書。
    ///
    /// 単語はJSONとの相互変換のために挿入された順序を保つ。
    #[napi(js_name = "UserDict")]
    pub struct JsUserDict {
        handle: UserDict,
    }

    fn parse_uuid(uuid: String) -> Result<Uuid> {
        Uuid::try_parse(&uuid).map_err(|err| Error::from_reason(err.to_string()))
    }

    #[napi]
    impl JsUserDict {
        /// ユーザー辞書を作成する。
        #[napi(constructor)]
        pub fn new() -> Self {
            JsUserDict {
                handle: UserDict::new(),
            }
        }

        /// ユーザー辞書をファイルから読み込む。
        ///
        /// @throws ファイルが読めなかった、または内容が不正だった場合はエラーを返す。
        #[napi]
        pub fn load(&self, store_path: String) -> Result<()> {
            convert_result(self.handle.load(&store_path))
        }

        /// ユーザー辞書に単語を追加する。
        #[napi]
        pub fn add_word(&self, word: UserDictWord) -> Result<String> {
            convert_result(self.handle.add_word(word.convert()?)).map(|uuid| uuid.to_string())
        }

        /// ユーザー辞書の単語を変更する。
        #[napi]
        pub fn update_word(&self, word_uuid: String, new_word: UserDictWord) -> Result<()> {
            convert_result(
                self.handle
                    .update_word(parse_uuid(word_uuid)?, new_word.convert()?),
            )
        }

        /// ユーザー辞書から単語を削除する。
        #[napi]
        pub fn remove_word(&self, word_uuid: String) -> Result<UserDictWord> {
            convert_result(self.handle.remove_word(parse_uuid(word_uuid)?))
                .map(|word| UserDictWord::from(word))
        }

        /// 他のユーザー辞書をインポートする。
        #[napi]
        pub fn import(&self, other: &JsUserDict) -> Result<()> {
            convert_result(self.handle.import(&other.handle))
        }

        /// ユーザー辞書を保存する。
        pub fn save(&self, store_path: String) -> Result<()> {
            convert_result(self.handle.save(&store_path))
        }
    }

    #[napi(js_name = "Synthesizer")]
    pub struct JsSynthesizer {
        handle: Synthesizer<OpenJtalk>,
    }

    #[napi]
    impl JsSynthesizer {
        /// `Synthesizer`をコンストラクトする。
        #[napi(constructor)]
        pub fn new(open_jtalk: &JsOpenJtalk, options: Option<InitializeOptions>) -> Result<Self> {
            Ok(JsSynthesizer {
                handle: convert_result(Synthesizer::new(
                    open_jtalk.handle.clone(),
                    &(options.unwrap_or_default().convert()?),
                ))?,
            })
        }

        /// ハードウェアアクセラレーションがGPUモードか判定する。
        #[napi]
        pub fn is_gpu_mode(&self) -> bool {
            self.handle.is_gpu_mode()
        }
    }
}

pub use blocking::{JsOpenJtalk, JsUserDict};
