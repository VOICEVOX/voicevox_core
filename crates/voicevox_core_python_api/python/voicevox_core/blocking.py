"""ブロッキング版API。"""

# pyright: reportMissingModuleSource=false
from ._rust.blocking import (
    Onnxruntime,
    OpenJtalk,
    SynthesisStream,
    Synthesizer,
    UserDict,
    VoiceModelFile,
)

__all__ = [
    "Onnxruntime",
    "OpenJtalk",
    "SynthesisStream",
    "Synthesizer",
    "UserDict",
    "VoiceModelFile",
]
