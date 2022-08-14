import dataclasses
from enum import Enum
from typing import List, Optional

import pydantic


@pydantic.dataclasses.dataclass
class Style:
    name: str
    id: int


@pydantic.dataclasses.dataclass
class Meta:
    name: str
    styles: List[Style]
    speaker_uuid: str
    version: str


@dataclasses.dataclass
class SupportedDevices:
    cpu: bool
    cuda: bool
    dml: bool


class AccelerationMode(str, Enum):
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
    speedScale: float
    pitchScale: float
    intonationScale: float
    volumeScale: float
    prePhonemeLength: float
    postPhonemeLength: float
    outputSamplingRate: int
    outputStereo: bool
    kana: Optional[str]
