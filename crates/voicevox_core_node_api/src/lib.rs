#![deny(clippy::all)]

#[napi]
pub mod blocking {
    use napi::Result;
    use voicevox_core::blocking::{OpenJtalk, UserDict};

    fn convert_result<T>(result: voicevox_core::Result<T>) -> napi::Result<T> {
        result.map_err(|err| napi::Error::from_reason(err.to_string()))
    }

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

    #[napi]
    impl JsUserDict {
        #[napi(constructor)]
        pub fn new() -> Self {
            JsUserDict {
                handle: UserDict::new(),
            }
        }
    }
}

pub mod devices;
pub mod model;

#[macro_use]
extern crate napi_derive;
