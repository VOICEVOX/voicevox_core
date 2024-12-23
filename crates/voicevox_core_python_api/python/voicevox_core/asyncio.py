# TODO: Rust API同様に、`$BLOCKING_MAX_THREADS`について言及
"""
非同期API。

Performance
-----------

未調査ではあるが、 ``cpu_num_threads`` 物理コアの数+1を指定するのが適切な可能性がある
(`VOICEVOX/voicevox_core#902 <https://github.com/VOICEVOX/voicevox_core/issues/902>`_)。
"""

# pyright: reportMissingModuleSource=false
from ._rust.asyncio import Onnxruntime, OpenJtalk, Synthesizer, UserDict, VoiceModelFile

__all__ = ["Onnxruntime", "OpenJtalk", "Synthesizer", "UserDict", "VoiceModelFile"]
