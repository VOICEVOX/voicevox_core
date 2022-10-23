from pathlib import Path
from typing import Final, List, Literal, Union

import numpy as np
from numpy.typing import NDArray

from voicevox_core import AccelerationMode, AudioQuery, Meta, SupportedDevices

METAS: Final[List[Meta]]
SUPPORTED_DEVICES: Final[SupportedDevices]
__version__: str

class VoicevoxCore:
    def __init__(
        self,
        acceleration_mode: Union[
            AccelerationMode, Literal["AUTO", "CPU", "GPU"]
        ] = AccelerationMode.AUTO,
        cpu_num_threads: int = 0,
        load_all_models: bool = False,
        open_jtalk_dict_dir: Union[Path, str, None] = None,
    ) -> None:
        """
        Parameters
        ----------
        acceleration_mode
            ハードウェアアクセラレーションモード。
        cpu_num_threads
            CPU利用数を指定。0を指定すると環境に合わせたCPUが利用される。
        load_all_models
            全てのモデルを読み込む。
        open_jtalk_dict_dir
            open_jtalkの辞書ディレクトリ。
        """
        ...
    def __repr__(self) -> str: ...
    @property
    def is_gpu_mode(self) -> bool:
        """ハードウェアアクセラレーションがGPUモードか判定する。

        Returns
        -------
        GPUモードならtrue、そうでないならfalse
        """
        ...
    def load_model(self, speaker_id: int) -> None:
        """モデルを読み込む。

        Parameters
        ----------
        speaker_id
            読み込むモデルの話者ID。
        """
        ...
    def is_model_loaded(self, speaker_id: int) -> bool:
        """指定したspeaker_idのモデルが読み込まれているか判定する。

        Returns
        -------
        モデルが読み込まれているのであればtrue、そうでないならfalse
        """
        ...
    def predict_duration(
        self,
        phoneme_vector: NDArray[np.int64],
        speaker_id: int,
    ) -> NDArray[np.float32]:
        """音素ごとの長さを推論する。

        Parameters
        ----------
        phoneme_vector
            音素データ。
        speaker_id
            話者ID。

        Returns
        -------
        音素ごとの長さ
        """
        ...
    def predict_intonation(
        self,
        length: int,
        vowel_phoneme_vector: NDArray[np.int64],
        consonant_phoneme_vector: NDArray[np.int64],
        start_accent_vector: NDArray[np.int64],
        end_accent_vector: NDArray[np.int64],
        start_accent_phrase_vector: NDArray[np.int64],
        end_accent_phrase_vector: NDArray[np.int64],
        speaker_id: int,
    ) -> NDArray[np.float32]:
        """モーラごとのF0を推論する。

        Parameters
        ----------
        length
            vowel_phoneme_vector, consonant_phoneme_vector, start_accent_vector, end_accent_vector, start_accent_phrase_vector, end_accent_phrase_vector, output のデータ長。
        vowel_phoneme_vector
            母音の音素データ。
        consonant_phoneme_vector
            子音の音素データ。
        start_accent_vector
            アクセントの開始位置のデータ。
        end_accent_vector
            アクセントの終了位置のデータ。
        start_accent_phrase_vector
            アクセント句の開始位置のデータ。
        end_accent_phrase_vector
            アクセント句の終了位置のデータ。
        speaker_id
            話者ID。

        Returns
        -------
        モーラごとのF0
        """
        ...
    def decode(
        self,
        length: int,
        phoneme_size: int,
        f0: NDArray[np.float32],
        phoneme: NDArray[np.float32],
        speaker_id: int,
    ) -> NDArray[np.float32]:
        """decodeを実行する。

        Parameters
        ----------
        length
            f0 , output のデータ長及び phoneme のデータ長に関連する。
        phoneme_size
            音素のサイズ phoneme のデータ長に関連する。
        f0
            基本周波数。
        phoneme_vector
            音素データ。
        speaker_id
            話者ID。

        Returns
        -------
        decode結果
        """
        ...
    def audio_query(
        self,
        text: str,
        speaker_id: int,
        kana: bool = False,
    ) -> AudioQuery:
        """AudioQuery を実行する。

        Parameters
        ----------
        text
            テキスト。
        speaker_id
            話者ID。
        kana
            aquestalk形式のkanaとしてテキストを解釈する。

        Returns
        -------
        :class:`AudioQuery`
        """
        ...
    def synthesis(
        self,
        audio_query: AudioQuery,
        speaker_id: int,
        enable_interrogative_upspeak: bool = True,
    ) -> bytes:
        """AudioQuery から音声合成する。

        Parameters
        ----------
        audio_query
            AudioQuery。
        speaker_id
            話者ID。
        enable_interrogative_upspeak
            疑問文の調整を有効にする。

        Returns
        -------
        wavデータ
        """
        ...
    def tts(
        self,
        text: str,
        speaker_id: int,
        kana: bool = False,
        enable_interrogative_upspeak: bool = True,
    ) -> bytes:
        """テキスト音声合成を実行する。

        Parameters
        ----------
        text
            テキスト。
        speaker_id
            話者ID。
        kana
            aquestalk形式のkanaとしてテキストを解釈する。
        enable_interrogative_upspeak
            疑問文の調整を有効にする。
        """
        ...
