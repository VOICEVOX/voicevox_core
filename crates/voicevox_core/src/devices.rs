use derive_getters::Getters;
use serde::{Deserialize, Serialize};

use crate::{infer::InferenceRuntime, synthesizer::InferenceRuntimeImpl, Result};

/// このライブラリで利用可能なデバイスの情報。
///
/// あくまで本ライブラリが対応しているデバイスの情報であることに注意。GPUが使える環境ではなかったと
/// しても`cuda`や`dml`は`true`を示しうる。
#[derive(Getters, Debug, Serialize, Deserialize)]
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
    /// `SupportedDevices`をコンストラクトする。
    ///
    /// # Example
    ///
    #[cfg_attr(windows, doc = "```no_run")] // https://github.com/VOICEVOX/voicevox_core/issues/537
    #[cfg_attr(not(windows), doc = "```")]
    /// use voicevox_core::SupportedDevices;
    ///
    /// let supported_devices = SupportedDevices::create()?;
    /// #
    /// # Result::<_, anyhow::Error>::Ok(())
    /// ```
    pub fn create() -> Result<Self> {
        <InferenceRuntimeImpl as InferenceRuntime>::supported_devices()
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).expect("should not fail")
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::SupportedDevices;

    #[rstest]
    fn supported_devices_create_works() {
        let result = SupportedDevices::create();
        // 環境によって結果が変わるので、関数呼び出しが成功するかどうかの確認のみ行う
        assert!(result.is_ok(), "{result:?}");
    }
}
