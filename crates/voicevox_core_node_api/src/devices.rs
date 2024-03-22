use napi::{Error, Result};

use voicevox_core::SupportedDevices;

/// このライブラリで利用可能なデバイスの情報。
///
/// あくまで本ライブラリが対応しているデバイスの情報であることに注意。GPUが使える環境ではなかったと
/// しても`cuda`や`dml`は`true`を示しうる。
#[napi(js_name = "SupportedDevices")]
pub struct JsSupportedDevices {
    supported_devices: SupportedDevices,
}

#[napi]
impl JsSupportedDevices {
    /// `SupportedDevices`をコンストラクトする。
    #[napi(factory)]
    pub fn create() -> Result<Self> {
        match SupportedDevices::create() {
            Ok(val) => Ok(JsSupportedDevices {
                supported_devices: val,
            }),
            Err(err) => Err(Error::from_reason(err.to_string())),
        }
    }

    /// CPUが利用可能。
    ///
    /// 常に`true`。
    #[napi(getter)]
    pub fn cpu(&self) -> bool {
        self.supported_devices.cpu
    }

    /// CUDAが利用可能。
    ///
    /// ONNX Runtimeの[CUDA Execution Provider] (`CUDAExecutionProvider`)に対応する。必要な環境につ
    /// いてはそちらを参照。
    ///
    /// [CUDA Execution Provider]: https://onnxruntime.ai/docs/execution-providers/CUDA-ExecutionProvider.html
    #[napi(getter)]
    pub fn cuda(&self) -> bool {
        self.supported_devices.cuda
    }

    /// DirectMLが利用可能。
    ///
    /// ONNX Runtimeの[DirectML Execution Provider] (`DmlExecutionProvider`)に対応する。必要な環境に
    /// ついてはそちらを参照。
    ///
    /// [DirectML Execution Provider]: https://onnxruntime.ai/docs/execution-providers/DirectML-ExecutionProvider.html
    #[napi(getter)]
    pub fn dml(&self) -> bool {
        self.supported_devices.dml
    }

    #[napi]
    pub fn to_json(&self) -> serde_json::Value {
        self.supported_devices.to_json()
    }
}
