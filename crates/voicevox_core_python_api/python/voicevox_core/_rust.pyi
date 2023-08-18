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

def supported_devices() -> SupportedDevices:
    """
    このライブラリで利用可能なデバイスの情報を取得する。

    .. code-block::

       import voicevox_core

       supported_devices = voicevox_core.supported_devices()
    """
    ...

class VoiceModel:
    """音声モデル。"""

    @staticmethod
    async def from_path(path: Union[Path, str]) -> "VoiceModel":
        """
        VVMファイルから ``VoiceModel`` を生成する。

        :param path: VVMファイルへのパス。
        """
        ...
    @property
    def id(self) -> str:
        """ID。"""
        ...
    @property
    def metas(self) -> List[SpeakerMeta]:
        """メタ情報。"""
        ...

class OpenJtalk:
    """
    テキスト解析器としてのOpen JTalk。

    :param open_jtalk_dict_dir: open_jtalkの辞書ディレクトリ。
    """

    def __init__(self, open_jtalk_dict_dir: Union[Path, str]) -> None: ...
    def use_user_dict(self, user_dict: UserDict) -> None:
        """ユーザー辞書を設定する。

        この関数を呼び出した後にユーザー辞書を変更した場合は、再度この関数を呼ぶ必要がある。

        Parameters
        ----------
        user_dict
            ユーザー辞書。
        """
        ...

class Synthesizer:
    """音声シンセサイザ。"""

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
        :class:`Synthesizer` を生成する。

        :param open_jtalk: Open JTalk。
        :param acceleration_mode: ハードウェアアクセラレーションモード。
        :param cpu_num_threads: CPU利用数を指定。0を指定すると環境に合わせたCPUが利用される。
        :param load_all_models: 全てのモデルを読み込む。
        """
        ...
    def __repr__(self) -> str: ...
    def __enter__(self) -> "Synthesizer": ...
    def __exit__(self, exc_type, exc_value, traceback) -> None: ...
    @property
    def is_gpu_mode(self) -> bool:
        """ハードウェアアクセラレーションがGPUモードかどうか。"""
        ...
    @property
    def metas(self) -> SpeakerMeta:
        """メタ情報。"""
        ...
    async def load_voice_model(self, model: VoiceModel) -> None:
        """
        モデルを読み込む。

        :param style_id: 読み込むモデルのスタイルID。
        """
        ...
    def unload_voice_model(self, voice_model_id: str) -> None:
        """音声モデルの読み込みを解除する。

        :param voice_model_id: 音声モデルID。
        """
        ...
    def is_loaded_voice_model(self, voice_model_id: str) -> bool:
        """
        指定したvoice_model_idのモデルが読み込まれているか判定する。

        :returns: モデルが読み込まれているかどうか。
        """
        ...
    async def audio_query(
        self,
        text: str,
        style_id: int,
        kana: bool = False,
    ) -> AudioQuery:
        """
        :class:`AudioQuery` を生成する。

        :param text: テキスト。文字コードはUTF-8。
        :param style_id: スタイルID。
        :param kana: ``text`` をAquesTalk風記法として解釈する。

        :returns: 話者とテキストから生成された :class:`AudioQuery` 。
        """
        ...
    async def create_accent_phrases(
        self,
        text: str,
        style_id: int,
        kana: bool = False,
    ) -> List[AccentPhrase]:
        """
        AccentPhrase (アクセント句)の配列を生成する。

        :param text: UTF-8の日本語テキストまたはAquesTalk風記法。
        :param style_id: スタイルID。
        :param kana: ``text`` をAquesTalk風記法として解釈する。

        :returns: :class:`AccentPhrase` の配列。
        """
        ...
    async def replace_mora_data(
        self,
        accent_phrases: List[AccentPhrase],
        style_id: int,
    ) -> List[AccentPhrase]:
        """アクセント句の音高・音素長を変更する。

        :param accent_phrases: 変更元のアクセント句。
        :param style_id: スタイルID。
        """
        ...
    async def replace_phoneme_length(
        self,
        accent_phrases: List[AccentPhrase],
        style_id: int,
    ) -> List[AccentPhrase]:
        """
        アクセント句の音素長を変更する。

        :param accent_phrases: 変更元のアクセント句。
        :param style_id: スタイルID。
        """
        ...
    async def replace_mora_pitch(
        self,
        accent_phrases: List[AccentPhrase],
        style_id: int,
    ) -> List[AccentPhrase]:
        """
        アクセント句の音高を変更する。

        :param accent_phrases: 変更元のアクセント句。
        :param style_id: スタイルID。
        """
        ...
    async def synthesis(
        self,
        audio_query: AudioQuery,
        style_id: int,
        enable_interrogative_upspeak: bool = True,
    ) -> bytes:
        """
        :class:`AudioQuery` から音声合成する。

        :param audio_query: :class:`AudioQuery` 。
        :param style_id: スタイルID。
        :param enable_interrogative_upspeak: 疑問文の調整を有効にする。

        :returns: WAVデータ。
        """
        ...
    async def tts(
        self,
        text: str,
        style_id: int,
        kana: bool = False,
        enable_interrogative_upspeak: bool = True,
    ) -> bytes:
        """
        テキスト音声合成を実行する。

        :param text: UTF-8の日本語テキストまたはAquesTalk風記法。
        :param style_id: スタイルID。
        :param kana: ``text`` をAquesTalk風記法として解釈する。
        :param enable_interrogative_upspeak: 疑問文の調整を有効にする。

        :returns: WAVデータ。
        """
        ...
    def close(self) -> None: ...

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
    """VOICEVOX COREのエラー。"""

    ...

def _validate_pronunciation(pronunciation: str) -> None: ...
def _to_zenkaku(text: str) -> str: ...
