use serde::{Deserialize, Serialize};

/// このライブラリで利用可能なデバイスの情報。
///
/// あくまで本ライブラリが対応しているデバイスの情報であることに注意。GPUが使える環境ではなかったと
/// しても`cuda`や`dml`は`true`を示しうる。
#[derive(Debug, Serialize, Deserialize)]
pub struct SupportedDevices {
    /// CPUが利用可能。
    ///
    /// 常に`true`。
    pub cpu: bool,
    /// CUDAが利用可能。
    ///
    /// ONNX Runtimeの[CUDA Execution Provider] (`CUDAExecutionProvider`)に対応する。必要な環境につ
    /// いてはそちらを参照。
    ///
    /// [CUDA Execution Provider]: https://onnxruntime.ai/docs/execution-providers/CUDA-ExecutionProvider.html
    pub cuda: bool,
    /// DirectMLが利用可能。
    ///
    /// ONNX Runtimeの[DirectML Execution Provider] (`DmlExecutionProvider`)に対応する。必要な環境に
    /// ついてはそちらを参照。
    ///
    /// [DirectML Execution Provider]: https://onnxruntime.ai/docs/execution-providers/DirectML-ExecutionProvider.html
    pub dml: bool,
}

impl SupportedDevices {
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).expect("should not fail")
    }
}
