from pathlib import Path
from typing import Dict, Final, List, Literal, Union
from uuid import UUID

import numpy as np
from numpy.typing import NDArray
from voicevox_core import (
    AccelerationMode,
    AccentPhrase,
    AudioQuery,
    SpeakerMeta,
    SupportedDevices,
    UserDict,
    UserDictWord,
)

__version__: str

def supported_devices() -> SupportedDevices: ...

class VoiceModel:
    @staticmethod
    async def from_path(path: Union[Path, str]) -> "VoiceModel":
        """
        Parameters
        ----------
        path
            vvmファイルへのパス
        """
        ...
    @property
    def id(self) -> str: ...
    @property
    def metas(self) -> List[SpeakerMeta]: ...

class OpenJtalk:
    def __init__(self, open_jtalk_dict_dir: Union[Path, str]) -> None:
        """
        Parameters
        ----------
        open_jtalk_dict_dir
            open_jtalkの辞書ディレクトリ。
        """
        ...
    def use_user_dict(self, user_dict: UserDict) -> None:
        """ユーザー辞書を設定する。

        この関数を読んだ後にユーザー辞書を変更した場合は、再度この関数を呼ぶ必要がある。

        Parameters
        ----------
        user_dict
            ユーザー辞書。
        """
        ...

class Synthesizer:
    @staticmethod
    async def new_with_initialize(
        open_jtalk: OpenJtalk,
        acceleration_mode: Union[
            AccelerationMode, Literal["AUTO", "CPU", "GPU"]
        ] = AccelerationMode.AUTO,
        cpu_num_threads: int = 0,
        load_all_models: bool = False,
    ) -> "Synthesizer":
        """
        Parameters
        ----------
        open_jtalk
        acceleration_mode
            ハードウェアアクセラレーションモード。
        cpu_num_threads
            CPU利用数を指定。0を指定すると環境に合わせたCPUが利用される。
        load_all_models
            全てのモデルを読み込む。
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
    @property
    def metas(self) -> SpeakerMeta:
        """メタ情報を取得する。"""
        ...
    async def load_voice_model(
        self, model: VoiceModel, gpu_num_sessions: int = 1
    ) -> None:
        """モデルを読み込む。

        Parameters
        ----------
        style_id
            読み込むモデルの話者ID。
        gpu_num_sessions
        """
        ...
    def unload_voice_model(self, voice_model_id: str) -> None:
        """モデルの読み込みを解除する。

        Parameters
        ----------
        voice_model_id
            音声モデルID。
        """
        ...
    def is_loaded_voice_model(self, voice_model_id: str) -> bool:
        """指定したvoice_model_idのモデルが読み込まれているか判定する。

        Returns
        -------
        モデルが読み込まれているのであればtrue、そうでないならfalse
        """
        ...
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
            テキスト。文字コードはUTF-8。
        style_id
            話者ID。
        kana
            aquestalk形式のkanaとしてテキストを解釈する。

        Returns
        -------
        :class:`List` [:class:`AccentPhrase`]
        """
        ...
    async def replace_mora_data(
        self,
        accent_phrases: List[AccentPhrase],
        style_id: int,
    ) -> List[AccentPhrase]:
        """アクセント句の音高・音素長を変更する。

        Parameters
        ----------
        accent_phrases
            変更元のアクセント句。
        style_id
            話者ID。
        Returns
        -------
        :class:`List` [:class:`AccentPhrase`]
        """
        ...
    async def replace_phoneme_length(
        self,
        accent_phrases: List[AccentPhrase],
        style_id: int,
    ) -> List[AccentPhrase]:
        """アクセント句の音素長を変更する。

        Parameters
        ----------
        accent_phrases
            変更元のアクセント句。
        style_id
            話者ID。
        Returns
        -------
        :class:`List` [:class:`AccentPhrase`]
        """
        ...
    async def replace_mora_pitch(
        self,
        accent_phrases: List[AccentPhrase],
        style_id: int,
    ) -> List[AccentPhrase]:
        """アクセント句の音高を変更する。

        Parameters
        ----------
        accent_phrases
            変更元のアクセント句。
        style_id
            話者ID。
        Returns
        -------
        :class:`List` [:class:`AccentPhrase`]
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

class UserDict:
    """ユーザー辞書。

    Attributes
    ----------
    words
        エントリーのリスト。
    """

    words: Dict[UUID, UserDictWord]
    def __init__(self) -> None:
        """ユーザー辞書をまたは新規作成する。"""
        ...
    def load(self, path: str) -> None:
        """ファイルに保存されたユーザー辞書を読み込む。

        Parameters
        ----------
        path
            ユーザー辞書のパス。
        """
        ...
    def save(self, path: str) -> None:
        """ユーザー辞書をファイルに保存する。

        Parameters
        ----------
        path
            ユーザー辞書のパス。
        """
        ...
    def add_word(self, word: UserDictWord) -> UUID:
        """単語を追加する。

        Parameters
        ----------
        word
            追加する単語。

        Returns
        -------
        単語のUUID。
        """
        ...
    def update_word(self, word_uuid: UUID, word: UserDictWord) -> None:
        """単語を更新する。

        Parameters
        ----------
        word_uuid
            更新する単語のUUID。
        word
            新しい単語のデータ。
        """
        ...
    def remove_word(self, word_uuid: UUID) -> None:
        """単語を削除する。

        Parameters
        ----------
        word_uuid
            削除する単語のUUID。
        """
        ...
    def import_dict(self, other: UserDict) -> None:
        """ユーザー辞書をインポートする。

        Parameters
        ----------
        other
            インポートするユーザー辞書。
        """
        ...

class VoicevoxError(Exception):
    """VOICEVOXで発生したエラー。"""

    ...

def _validate_pronunciation(pronunciation: str) -> None: ...
def _to_zenkaku(text: str) -> str: ...
