from pathlib import Path
from typing import Final, List, Literal, Union

import numpy as np
from numpy.typing import NDArray

from voicevox_core import (
    AccelerationMode,
    AudioQuery,
    SpeakerMeta,
    SupportedDevices,
    AccentPhrase,
)

__version__: str

def supported_devices() -> SupportedDevices: ...

class VoiceModel:
    @staticmethod
    async def from_path(path: str) -> VoiceModel:
        """
        Parameters
        ----------
        path
            vvmファイルへのパス
        """
        ...
    def id() -> str: ...
    def metas() -> List[SpeakerMeta]: ...

class Synthesizer:
    @staticmethod
    async def new_with_initialize(
        self,
        acceleration_mode: Union[
            AccelerationMode, Literal["AUTO", "CPU", "GPU"]
        ] = AccelerationMode.AUTO,
        cpu_num_threads: int = 0,
        load_all_models: bool = False,
        open_jtalk_dict_dir: Union[Path, str, None] = None,
    ) -> VoicevoxSynthesizer:
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
    async def load_voice_model(self, model: VoiceModel) -> None:
        """モデルを読み込む。

        Parameters
        ----------
        style_id
            読み込むモデルの話者ID。
        """
        ...
    def is_loaded_voice_model(self, voice_model_id: str) -> bool:
        """指定したvoice_model_idのモデルが読み込まれているか判定する。

        Returns
        -------
        モデルが読み込まれているのであればtrue、そうでないならfalse
        """
        ...
    def unload_voice_model(self, voice_model_id: str) -> None:
        """指定したvoice_model_idのモデルがを破棄する"""
    async def audio_query(
        self,
        text: str,
        style_id: int,
        kana: bool = False,
    ) -> AudioQuery:
        """AudioQuery を実行する。

        Parameters
        ----------
        text
            テキスト。文字コードはUTF-8。
        style_id
            話者ID。
        kana
            aquestalk形式のkanaとしてテキストを解釈する。

        Returns
        -------
        :class:`AudioQuery`
        """
        ...
    async def create_accent_phrases(
        self,
        text: str,
        style_id: int,
        kana: bool = False,
    ) -> List[AccentPhrase]:
        """create_accent_phrases を実行する。

        Parameters
        ----------
        text
            テキスト。
        style_id
            話者ID。
        kana
            aquestalk形式のkanaとしてテキストを解釈する。

        Returns
        -------
        :class:`AudioQuery`
        """
        ...
    async def replace_mora_data(
        self,
        accent_phrases: List[AccentPhrase],
        style_id: int,
    ) -> List[AccentPhrase]:
        """replace_mora_data を実行する。

        Parameters
        ----------
        accent_phrases
            AccentPhraseのリスト
        style_id
            話者ID。
        Returns
        -------
        :class:`AudioQuery`
        """
        ...
    async def replace_phoneme_length(
        self,
        accent_phrases: List[AccentPhrase],
        style_id: int,
    ) -> List[AccentPhrase]:
        """replace_phoneme_length を実行する。

        Parameters
        ----------
        accent_phrases
            AccentPhraseのリスト
        style_id
            話者ID。
        Returns
        -------
        :class:`AudioQuery`
        """
        ...
    async def synthesis(
        self,
        audio_query: AudioQuery,
        style_id: int,
        enable_interrogative_upspeak: bool = True,
    ) -> bytes:
        """AudioQuery から音声合成する。

        Parameters
        ----------
        audio_query
            AudioQuery。
        style_id
            話者ID。
        enable_interrogative_upspeak
            疑問文の調整を有効にする。

        Returns
        -------
        wavデータ
        """
        ...
    async def tts(
        self,
        text: str,
        style_id: int,
        kana: bool = False,
        enable_interrogative_upspeak: bool = True,
    ) -> bytes:
        """テキスト音声合成を実行する。

        Parameters
        ----------
        text
            テキスト。文字コードはUTF-8。
        style_id
            話者ID。
        kana
            aquestalk形式のkanaとしてテキストを解釈する。
        enable_interrogative_upspeak
            疑問文の調整を有効にする。
        """
        ...
