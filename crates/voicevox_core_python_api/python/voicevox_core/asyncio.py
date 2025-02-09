"""
非同期API。

Performance
-----------

内部では `Rustのblockingというライブラリ <https://docs.rs/crate/blocking>`_
を用いている。そのため ``$BLOCKING_MAX_THREADS``
から内部のスレッドプールのサイズを調整可能である。

また未調査ではあるが、 ``cpu_num_threads``
は物理コアの数+1を指定するのが適切な可能性がある
(`VOICEVOX/voicevox_core#902 <https://github.com/VOICEVOX/voicevox_core/issues/902>`_)。
"""

# pyright: reportMissingModuleSource=false
from ._rust.asyncio import Onnxruntime, OpenJtalk, Synthesizer, UserDict, VoiceModelFile

__all__ = ["Onnxruntime", "OpenJtalk", "Synthesizer", "UserDict", "VoiceModelFile"]
