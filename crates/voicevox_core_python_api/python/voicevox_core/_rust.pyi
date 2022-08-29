from pathlib import Path
from typing import Final, List, Literal, Union

import numpy as np
from numpy.typing import NDArray

from voicevox_core import AccelerationMode, AudioQuery, Meta, SupportedDevices

METAS: Final[List[Meta]]
SUPPORTED_DEVICES: Final[SupportedDevices]

class VoicevoxCore:
    def __init__(
        self,
        acceleration_mode: Union[
            AccelerationMode, Literal["AUTO", "CPU", "GPU"], None
        ] = None,
        cpu_num_threads: int = 0,
        load_all_models: bool = False,
        open_jtalk_dict_dir: Union[Path, str, None] = None,
    ) -> None:
        pass
    def __repr__(self) -> str:
        pass
    @property
    def is_gpu_mode(self) -> bool:
        pass
    def load_model(self, speaker_id: int) -> None:
        pass
    def is_model_loaded(self, speaker_id: int) -> bool:
        pass
    def predict_duration(
        self,
        phoneme_list: NDArray[np.int64],
        speaker_id: int,
    ) -> NDArray[np.float32]:
        pass
    def predict_intonation(
        self,
        length: int,
        vowel_phoneme_list: NDArray[np.int64],
        consonant_phoneme_list: NDArray[np.int64],
        start_accent_list: NDArray[np.int64],
        end_accent_list: NDArray[np.int64],
        start_accent_phrase_list: NDArray[np.int64],
        end_accent_phrase_list: NDArray[np.int64],
        speaker_id: int,
    ) -> NDArray[np.float32]:
        pass
    def decode(
        self,
        length: int,
        phoneme_size: int,
        f0: NDArray[np.float32],
        phoneme: NDArray[np.float32],
        speaker_id: int,
    ):
        pass
    def audio_query(
        self,
        text: str,
        speaker_id: int,
        kana: bool = False,
    ) -> AudioQuery:
        pass
    def synthesis(
        self,
        audio_query: AudioQuery,
        speaker_id: int,
        enable_interrogative_upspeak: bool = True,
    ) -> bytes:
        pass
    def tts(
        self,
        text: str,
        speaker_id: int,
        kana: bool = False,
        enable_interrogative_upspeak: bool = True,
    ) -> bytes:
        pass
