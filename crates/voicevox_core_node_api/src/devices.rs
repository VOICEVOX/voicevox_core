use napi::{Error, Result};

use voicevox_core::SupportedDevices;

#[napi(js_name = "SupportedDevices")]
pub struct JsSupportedDevices {
    handle: SupportedDevices,
}

#[napi]
impl JsSupportedDevices {
    #[napi(factory)]
    pub fn create() -> Result<Self> {
        match SupportedDevices::create() {
            Ok(val) => Ok(JsSupportedDevices { handle: val }),
            Err(err) => Err(Error::from_reason(err.to_string())),
        }
    }

    #[napi(getter)]
    pub fn cpu(&self) -> bool {
        self.handle.cpu
    }

    #[napi(getter)]
    pub fn cuda(&self) -> bool {
        self.handle.cuda
    }

    #[napi(getter)]
    pub fn dml(&self) -> bool {
        self.handle.dml
    }

    #[napi]
    pub fn to_json(&self) -> serde_json::Value {
        self.handle.to_json()
    }
}
