from pathlib import Path
from typing import TYPE_CHECKING, Dict, List, Literal, Union
from uuid import UUID

if TYPE_CHECKING:
    from voicevox_core import (
        AccelerationMode,
        AccentPhrase,
        AudioQuery,
        SpeakerMeta,
        StyleId,
        SupportedDevices,
        UserDict,
        UserDictWord,
        VoiceModelId,
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
    """
    音声モデル。"""

    @staticmethod
    async def from_path(path: Union[Path, str]) -> VoiceModel:
        """
        VVMファイルから ``VoiceModel`` を生成する。

        Parameters
        ----------
        path
            VVMファイルへのパス。
        """
        ...
    @property
    def id(self) -> VoiceModelId:
        """ID。"""
        ...
    @property
    def metas(self) -> List[SpeakerMeta]:
        """メタ情報。"""
        ...

class OpenJtalk:
    """
    テキスト解析器としてのOpen JTalk。
    """

    @staticmethod
    async def new(open_jtalk_dict_dir: Union[Path, str]) -> "OpenJtalk":
        """
        ``OpenJTalk`` を生成する。

        Parameters
        ----------
        open_jtalk_dict_dir
            Open JTalkの辞書ディレクトリ。
        """
        ...
    async def use_user_dict(self, user_dict: UserDict) -> None:
        """
        ユーザー辞書を設定する。

        この関数を呼び出した後にユーザー辞書を変更した場合は、再度この関数を呼ぶ必要がある。

        Parameters
        ----------
        user_dict
            ユーザー辞書。
        """
        ...

class Synthesizer:
    """
    音声シンセサイザ。

    Parameters
    ----------
    open_jtalk
        Open JTalk。
    acceleration_mode
        ハードウェアアクセラレーションモード。
    cpu_num_threads
        CPU利用数を指定。0を指定すると環境に合わせたCPUが利用される。
    """

    def __init__(
        self,
        open_jtalk: OpenJtalk,
        acceleration_mode: Union[
            AccelerationMode, Literal["AUTO", "CPU", "GPU"]
        ] = AccelerationMode.AUTO,
        cpu_num_threads: int = 0,
    ) -> None: ...
    def __repr__(self) -> str: ...
    def __enter__(self) -> "Synthesizer": ...
    def __exit__(self, exc_type, exc_value, traceback) -> None: ...
    @property
    def is_gpu_mode(self) -> bool:
        """ハードウェアアクセラレーションがGPUモードかどうか。"""
        ...
    @property
    def metas(self) -> List[SpeakerMeta]:
        """メタ情報。"""
        ...
    async def load_voice_model(self, model: VoiceModel) -> None:
        """
        モデルを読み込む。

        Parameters
        ----------
        style_id
            読み込むモデルのスタイルID。
        """
        ...
    def unload_voice_model(self, voice_model_id: Union[VoiceModelId, str]) -> None:
        """
        音声モデルの読み込みを解除する。

        Parameters
        ----------
        voice_model_id
            音声モデルID。
        """
        ...
    def is_loaded_voice_model(self, voice_model_id: Union[VoiceModelId, str]) -> bool:
        """
        指定したvoice_model_idのモデルが読み込まれているか判定する。

        Parameters
        ----------
        voice_model_id
            音声モデルID。

        Returns
        -------
        モデルが読み込まれているかどうか。
        """
        ...
    async def audio_query_from_kana(
        self,
        kana: str,
        style_id: Union[StyleId, int],
    ) -> AudioQuery:
        """
        AquesTalk風記法から :class:`AudioQuery` を生成する。

        Parameters
        ----------
        kana
            AquesTalk風記法。
        style_id
            スタイルID。

        Returns
        -------
        話者とテキストから生成された :class:`AudioQuery` 。
        """
        ...
    async def audio_query(
        self,
        text: str,
        style_id: Union[StyleId, int],
    ) -> AudioQuery:
        """
        日本語のテキストから :class:`AudioQuery` を生成する。

        Parameters
        ----------
        text
            UTF-8の日本語テキスト。
        style_id
            スタイルID。

        Returns
        -------
        話者とテキストから生成された :class:`AudioQuery` 。
        """
        ...
    async def create_accent_phrases_from_kana(
        self,
        kana: str,
        style_id: Union[StyleId, int],
    ) -> List[AccentPhrase]:
        """
        AquesTalk風記法からAccentPhrase（アクセント句）の配列を生成する。

        Parameters
        ----------
        kana
            AquesTalk風記法。
        style_id
            スタイルID。

        Returns
        -------
        :class:`AccentPhrase` の配列。
        """
        ...
    async def create_accent_phrases(
        self,
        text: str,
        style_id: Union[StyleId, int],
    ) -> List[AccentPhrase]:
        """
        日本語のテキストからAccentPhrase（アクセント句）の配列を生成する。

        Parameters
        ----------
        text
            UTF-8の日本語テキスト。
        style_id
            スタイルID。

        Returns
        -------
        :class:`AccentPhrase` の配列。
        """
        ...
    async def replace_mora_data(
        self,
        accent_phrases: List[AccentPhrase],
        style_id: Union[StyleId, int],
    ) -> List[AccentPhrase]:
        """
        アクセント句の音高・音素長を変更した新しいアクセント句の配列を生成する。

        元のアクセント句の音高・音素長は変更されない。

        Parameters
        ----------
        accent_phrases:
            変更元のアクセント句。
        style_id:
            スタイルID。

        Returns
        -------
        新しいアクセント句の配列。
        """
        ...
    async def replace_phoneme_length(
        self,
        accent_phrases: List[AccentPhrase],
        style_id: Union[StyleId, int],
    ) -> List[AccentPhrase]:
        """
        アクセント句の音素長を変更した新しいアクセント句の配列を生成する。

        元のアクセント句の音素長は変更されない。

        Parameters
        ----------
        accent_phrases
            変更元のアクセント句。
        style_id
            スタイルID。
        """
        ...
    async def replace_mora_pitch(
        self,
        accent_phrases: List[AccentPhrase],
        style_id: Union[StyleId, int],
    ) -> List[AccentPhrase]:
        """
        アクセント句の音高を変更した新しいアクセント句の配列を生成する。

        元のアクセント句の音高は変更されない。

        Parameters
        ----------
        accent_phrases
            変更元のアクセント句。
        style_id
            スタイルID。
        """
        ...
    async def synthesis(
        self,
        audio_query: AudioQuery,
        style_id: Union[StyleId, int],
        enable_interrogative_upspeak: bool = True,
    ) -> bytes:
        """
        :class:`AudioQuery` から音声合成する。

        Parameters
        ----------
        audio_query
            :class:`AudioQuery` 。
        style_id
            スタイルID。
        enable_interrogative_upspeak
            疑問文の調整を有効にするかどうか。

        Returns
        -------
        WAVデータ。
        """
        ...
    async def tts_from_kana(
        self,
        kana: str,
        style_id: Union[StyleId, int],
        enable_interrogative_upspeak: bool = True,
    ) -> bytes:
        """
        AquesTalk風記法から音声合成を行う。

        Parameters
        ----------
        kana
            AquesTalk風記法。
        style_id
            スタイルID。
        enable_interrogative_upspeak
            疑問文の調整を有効にするかどうか。
        """
        ...
    async def tts(
        self,
        text: str,
        style_id: Union[StyleId, int],
        enable_interrogative_upspeak: bool = True,
    ) -> bytes:
        """
        日本語のテキストから音声合成を行う。

        Parameters
        ----------
        text
            UTF-8の日本語テキスト。
        style_id
            スタイルID。
        enable_interrogative_upspeak
            疑問文の調整を有効にするかどうか。

        Returns
        -------
        WAVデータ。
        """
        ...
    def close(self) -> None: ...

class UserDict:
    """ユーザー辞書。"""

    @property
    def words(self) -> Dict[UUID, UserDictWord]:
        """このオプジェクトの :class:`dict` としての表現。"""
        ...
    def __init__(self) -> None: ...
    async def load(self, path: str) -> None:
        """ファイルに保存されたユーザー辞書を読み込む。

        Parameters
        ----------
        path
            ユーザー辞書のパス。
        """
        ...
    async def save(self, path: str) -> None:
        """
        ユーザー辞書をファイルに保存する。

        Parameters
        ----------
        path
            ユーザー辞書のパス。
        """
        ...
    def add_word(self, word: UserDictWord) -> UUID:
        """
        単語を追加する。

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
        """
        単語を更新する。

        Parameters
        ----------
        word_uuid
            更新する単語のUUID。
        word
            新しい単語のデータ。
        """
        ...
    def remove_word(self, word_uuid: UUID) -> None:
        """
        単語を削除する。

        Parameters
        ----------
        word_uuid
            削除する単語のUUID。
        """
        ...
    def import_dict(self, other: UserDict) -> None:
        """
        ユーザー辞書をインポートする。

        Parameters
        ----------
        other
            インポートするユーザー辞書。
        """
        ...

class NotLoadedOpenjtalkDictError(Exception):
    """open_jtalk辞書ファイルが読み込まれていない。"""

    ...

class GpuSupportError(Exception):
    """GPUモードがサポートされていない。"""

    ...

class OpenZipFileError(Exception):
    """ZIPファイルを開くことに失敗した。"""

    ...

class ReadZipEntryError(Exception):
    """ZIP内のファイルが読めなかった。"""

    ...

class ModelAlreadyLoadedError(Exception):
    """すでに読み込まれている音声モデルを読み込もうとした。"""

    ...

class StyleAlreadyLoadedError(Exception):
    """すでに読み込まれているスタイルを読み込もうとした。"""

    ...

class InvalidModelDataError(Exception):
    """無効なモデルデータ。"""

    ...

class GetSupportedDevicesError(Exception):
    """サポートされているデバイス情報取得に失敗した。"""

    ...

class StyleNotFoundError(KeyError):
    """スタイルIDに対するスタイルが見つからなかった。"""

    ...

class ModelNotFoundError(KeyError):
    """音声モデルIDに対する音声モデルが見つからなかった。"""

    ...

class InferenceFailedError(Exception):
    """推論に失敗した。"""

    ...

class ExtractFullContextLabelError(Exception):
    """コンテキストラベル出力に失敗した。"""

    ...

class ParseKanaError(ValueError):
    """AquesTalk風記法のテキストの解析に失敗した。"""

    ...

class LoadUserDictError(Exception):
    """ユーザー辞書を読み込めなかった。"""

    ...

class SaveUserDictError(Exception):
    """ユーザー辞書を書き込めなかった。"""

    ...

class WordNotFoundError(KeyError):
    """ユーザー辞書に単語が見つからなかった。"""

    ...

class UseUserDictError(Exception):
    """OpenJTalkのユーザー辞書の設定に失敗した。"""

    ...

class InvalidWordError(ValueError):
    """ユーザー辞書の単語のバリデーションに失敗した。"""

    ...

def _validate_pronunciation(pronunciation: str) -> None: ...
def _to_zenkaku(text: str) -> str: ...
