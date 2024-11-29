use std::{
    collections::BTreeMap,
    fmt::{self, Display},
    ops::Index,
};

use derive_more::BitAnd;
use serde::{Deserialize, Serialize};

pub(crate) fn test_gpus(
    gpus: impl IntoIterator<Item = GpuSpec>,
    inference_rt_name: &'static str,
    devices_supported_by_inference_rt: SupportedDevices,
    test: impl Fn(GpuSpec) -> anyhow::Result<()>,
) -> DeviceAvailabilities {
    DeviceAvailabilities(
        gpus.into_iter()
            .map(|gpu| {
                let availability = test_gpu(
                    gpu,
                    inference_rt_name,
                    devices_supported_by_inference_rt,
                    &test,
                );
                (gpu, availability)
            })
            .collect(),
    )
}

fn test_gpu(
    gpu: GpuSpec,
    inference_rt_name: &'static str,
    devices_supported_by_inference_rt: SupportedDevices,
    test: impl Fn(GpuSpec) -> anyhow::Result<()>,
) -> DeviceAvailability {
    if !SupportedDevices::THIS[gpu] {
        DeviceAvailability::NotSupportedByThisLib
    } else if !devices_supported_by_inference_rt[gpu] {
        DeviceAvailability::NotSupportedByCurrentLoadedInferenceRuntime(inference_rt_name)
    } else {
        match test(gpu) {
            Ok(()) => DeviceAvailability::Ok,
            Err(err) => DeviceAvailability::Err(err),
        }
    }
}

/// 利用可能なデバイスの情報。
///
/// あくまで本ライブラリもしくはONNX Runtimeが対応しているデバイスの情報であることに注意。GPUが使える環境ではなかったと
/// しても`cuda`や`dml`は`true`を示しうる。
///
/// ```
/// # #[pollster::main]
/// # async fn main() -> anyhow::Result<()> {
/// use voicevox_core::{nonblocking::Onnxruntime, SupportedDevices};
///
/// # voicevox_core::blocking::Onnxruntime::load_once()
/// #     .filename(if cfg!(windows) {
/// #         // Windows\System32\onnxruntime.dllを回避
/// #         test_util::ONNXRUNTIME_DYLIB_PATH
/// #     } else {
/// #         voicevox_core::blocking::Onnxruntime::LIB_VERSIONED_FILENAME
/// #     })
/// #     .exec()?;
/// #
/// let onnxruntime = Onnxruntime::get().unwrap();
/// dbg!(SupportedDevices::THIS & onnxruntime.supported_devices()?);
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Debug, BitAnd, Serialize, Deserialize)]
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
    /// このライブラリで利用可能なデバイスの情報。
    ///
    /// `load-onnxruntime`のフィーチャが有効化されているときはすべて`true`となる。
    ///
    #[cfg_attr(feature = "load-onnxruntime", doc = "```")]
    #[cfg_attr(not(feature = "load-onnxruntime"), doc = "```no_run")]
    /// # use voicevox_core::SupportedDevices;
    /// assert!(SupportedDevices::THIS.cuda);
    /// assert!(SupportedDevices::THIS.dml);
    /// ```
    ///
    /// `link-onnxruntime`のフィーチャが有効化されているときは`cpu`を除き`false`となる。
    ///
    #[cfg_attr(feature = "link-onnxruntime", doc = "```")]
    #[cfg_attr(not(feature = "link-onnxruntime"), doc = "```no_run")]
    /// # use voicevox_core::SupportedDevices;
    /// assert!(!SupportedDevices::THIS.cuda);
    /// assert!(!SupportedDevices::THIS.dml);
    /// ```
    pub const THIS: Self = if cfg!(feature = "load-onnxruntime") {
        Self {
            cpu: true,
            cuda: true,
            dml: true,
        }
    } else if cfg!(feature = "link-onnxruntime") {
        Self {
            cpu: true,
            cuda: false,
            dml: false,
        }
    } else {
        panic!("either `load-onnxruntime` or `link-onnxruntime` must be enabled");
    };

    pub fn to_json(self) -> serde_json::Value {
        serde_json::to_value(self).expect("should not fail")
    }
}

#[derive(Debug)]
pub(crate) struct DeviceAvailabilities(BTreeMap<GpuSpec, DeviceAvailability>);

impl DeviceAvailabilities {
    pub(crate) fn oks(&self) -> Vec<GpuSpec> {
        self.0
            .iter()
            .filter(|(_, result)| matches!(result, DeviceAvailability::Ok))
            .map(|(&gpu, _)| gpu)
            .collect()
    }
}

impl Display for DeviceAvailabilities {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (gpu, availability) in &self.0 {
            match availability {
                DeviceAvailability::Ok => writeln!(f, "* {gpu}: OK"),
                DeviceAvailability::Err(err) => {
                    writeln!(f, "* {gpu}: {err}", err = err.to_string().trim_end())
                }
                DeviceAvailability::NotSupportedByThisLib => {
                    writeln!(
                        f,
                        "* {gpu}: この`{name}`のビルドでは利用できません",
                        name = env!("CARGO_PKG_NAME"),
                    )
                }
                DeviceAvailability::NotSupportedByCurrentLoadedInferenceRuntime(name) => {
                    writeln!(f, "* {gpu}: {name}では利用できません")
                }
            }?;
        }
        Ok(())
    }
}

#[derive(Debug)]
enum DeviceAvailability {
    Ok,
    Err(anyhow::Error),
    NotSupportedByThisLib,
    NotSupportedByCurrentLoadedInferenceRuntime(&'static str),
}

#[derive(Clone, Copy, PartialEq, Debug, derive_more::Display)]
pub(crate) enum DeviceSpec {
    #[display(fmt = "CPU")]
    Cpu,

    #[display(fmt = "{_0}")]
    Gpu(GpuSpec),
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, derive_more::Display)]
pub(crate) enum GpuSpec {
    #[display(fmt = "CUDA (device_id=0)")]
    Cuda,

    #[display(fmt = "DirectML (device_id=0)")]
    Dml,
}

impl GpuSpec {
    pub(crate) fn defaults() -> Vec<Self> {
        vec![Self::Cuda, Self::Dml]
    }
}

impl Index<GpuSpec> for SupportedDevices {
    type Output = bool;

    fn index(&self, gpu: GpuSpec) -> &Self::Output {
        match gpu {
            GpuSpec::Cuda => &self.cuda,
            GpuSpec::Dml => &self.dml,
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::{GpuSpec, SupportedDevices};

    #[test]
    fn gpu_spec_defaults_is_exhaustive() {
        static SUPPORTED_DEVICES: SupportedDevices = SupportedDevices::THIS; // whatever

        assert_eq!(
            {
                #[forbid(
                    unused_variables,
                    reason = "比較対象としてここは網羅されてなければなりません"
                )]
                let SupportedDevices { cpu: _, cuda, dml } = &SUPPORTED_DEVICES;
                [&raw const *cuda, &raw const *dml]
            },
            *GpuSpec::defaults()
                .into_iter()
                .map(|gpu| &raw const SUPPORTED_DEVICES[gpu])
                .collect::<Vec<_>>(),
        );
    }
}
