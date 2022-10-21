from enum import Enum
from typing import List, Optional

import pydantic


@pydantic.dataclasses.dataclass
class Style:
    name: str
    id: int


@pydantic.dataclasses.dataclass
class Meta:
    """メタ情報。"""

    name: str
    styles: List[Style]
    speaker_uuid: str
    version: str


@pydantic.dataclasses.dataclass
class SupportedDevices:
    """サポートデバイス情報。"""

    cpu: bool
    cuda: bool
    dml: bool


class AccelerationMode(str, Enum):
    """
    ハードウェアアクセラレーションモードを設定する設定値。
    """

    AUTO = "AUTO"
    CPU = "CPU"
    GPU = "GPU"


@pydantic.dataclasses.dataclass
class Mora:
    text: str
    consonant: Optional[str]
    consonant_length: Optional[float]
    vowel: str
    vowel_length: float
    pitch: float


@pydantic.dataclasses.dataclass
class AccentPhrase:
    moras: List[Mora]
    accent: int
    pause_mora: Optional[Mora]
    is_interrogative: bool


@pydantic.dataclasses.dataclass
class AudioQuery:
    accent_phrases: List[AccentPhrase]
    speed_scale: float
    pitch_scale: float
    intonation_scale: float
    volume_scale: float
    pre_phoneme_length: float
    post_phoneme_length: float
    output_sampling_rate: int
    output_stereo: bool
    kana: Optional[str]
