"""無料で使える中品質なテキスト読み上げソフトウェア、VOICEVOXのコア。"""

from . import _load_dlls  # noqa: F401
from ._models import (  # noqa: F401
    AccelerationMode,
    AccentPhrase,
    AudioQuery,
    Mora,
    SpeakerMeta,
    SupportedDevices,
    UserDictWord,
    UserDictWordType,
)
from ._rust import (  # noqa: F401
    OpenJtalk,
    Synthesizer,
    UserDict,
    VoiceModel,
    VoicevoxError,
    __version__,
    supported_devices,
)

__all__ = [
    "__version__",
    "AccelerationMode",
    "AccentPhrase",
    "AudioQuery",
    "Mora",
    "OpenJtalk",
    "SpeakerMeta",
    "SupportedDevices",
    "Synthesizer",
    "VoicevoxError",
    "VoiceModel",
    "supported_devices",
    "UserDict",
    "UserDictWord",
    "UserDictWordType",
]
