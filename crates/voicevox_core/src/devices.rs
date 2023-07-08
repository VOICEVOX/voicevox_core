use serde::{Deserialize, Serialize};

use super::*;

/// このライブラリで利用可能なデバイスの情報。
///
/// あくまで本ライブラリが対応しているデバイスの情報であることに注意。GPUが使える環境ではなかったと
/// しても`cuda`や`dml`は`true`を示しうる。
#[derive(Getters, Debug, Serialize, Deserialize)]
pub struct SupportedDevices {
    /// CPUが利用可能。
    ///
    /// 常に`true`。
    cpu: bool,
    /// [CUDA Execution Provider] (`CUDAExecutionProvider`)が利用可能。
    ///
    /// [CUDA Execution Provider]: https://onnxruntime.ai/docs/execution-providers/CUDA-ExecutionProvider.html
    cuda: bool,
    /// [DirectML Execution Provider] (`DmlExecutionProvider`)が利用可能。
    ///
    /// [DirectML Execution Provider]: https://onnxruntime.ai/docs/execution-providers/DirectML-ExecutionProvider.html
    dml: bool,
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
        let mut cuda_support = false;
        let mut dml_support = false;
        for provider in onnxruntime::session::get_available_providers()
            .map_err(|e| Error::GetSupportedDevices(e.into()))?
            .iter()
        {
            match provider.as_str() {
                "CUDAExecutionProvider" => cuda_support = true,
                "DmlExecutionProvider" => dml_support = true,
                _ => {}
            }
        }

        Ok(SupportedDevices {
            cpu: true,
            cuda: cuda_support,
            dml: dml_support,
        })
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).expect("should not fail")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[rstest]
    fn supported_devices_create_works() {
        let result = SupportedDevices::create();
        // 環境によって結果が変わるので、関数呼び出しが成功するかどうかの確認のみ行う
        assert!(result.is_ok(), "{result:?}");
    }
}
