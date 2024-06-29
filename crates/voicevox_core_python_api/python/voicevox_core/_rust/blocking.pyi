from os import PathLike
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
        UserDictWord,
        VoiceModelId,
    )

class VoiceModel:
    """
    音声モデル。"""

    @staticmethod
    def from_path(path: Union[str, PathLike[str]]) -> VoiceModel:
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

class Onnxruntime:
    """
    ONNX Runtime。

    シングルトンであり、インスタンスは高々一つ。

    .. code-block::

        ort1 = Onnxruntime.load_once()
        ort2 = Onnxruntime.get()
        assert ort2
        assert ort2 is ort1

    .. code-block::

        ort = voicevox_core.blocking.Onnxruntime.load_once()
        assert voicevox_core.asyncio.Onnxruntime.get()
    """

    # ここの定数値が本物と合致するかどうかは、test_type_stub_consts.pyで担保する。

    LIB_NAME: str = "onnxruntime"
    """ONNX Runtimeのライブラリ名。"""

    LIB_VERSION: str = "1.17.3"
    """推奨されるONNX Runtimeのバージョン。"""

    LIB_VERSIONED_FILENAME: str
    """
    :attr:`LIB_NAME` と :attr:`LIB_VERSION` からなる動的ライブラリのファイル名。

    WindowsとAndroidでは :attr:`LIB_UNVERSIONED_FILENAME` と同じ。
    """

    LIB_UNVERSIONED_FILENAME: str
    """:attr:`LIB_NAME` からなる動的ライブラリのファイル名。"""

    @staticmethod
    def get() -> Union["Onnxruntime", None]:
        """
        インスタンスが既に作られているならそれを得る。

        作られていなければ ``None`` を返す。
        """
        ...
    @staticmethod
    def load_once(*, filename: str = LIB_VERSIONED_FILENAME) -> "Onnxruntime":
        """
        ONNX Runtimeをロードして初期化する。

        一度成功したら、以後は引数を無視して同じインスタンスを返す。

        Parameters
        ----------
        filename
            ``dlopen``/``LoadLibraryExW`` の引数に使われるfilename。
        """
        ...
    def supported_devices(self) -> SupportedDevices:
        """
        このライブラリで利用可能なデバイスの情報を取得する。
        """
        ...

class OpenJtalk:
    """
    テキスト解析器としてのOpen JTalk。

    Parameters
    ----------
    open_jtalk_dict_dir
        Open JTalkの辞書ディレクトリ。
    """

    def __init__(self, open_jtalk_dict_dir: Union[str, PathLike[str]]) -> None: ...
    def use_user_dict(self, user_dict: UserDict) -> None:
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
    onnxruntime
        ONNX Runtime。
    open_jtalk
        Open JTalk。
    acceleration_mode
        ハードウェアアクセラレーションモード。
    cpu_num_threads
        CPU利用数を指定。0を指定すると環境に合わせたCPUが利用される。
    """

    def __init__(
        self,
        onnxruntime: Onnxruntime,
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
    def onnxruntime(self) -> Onnxruntime:
        """ONNX Runtime。"""
        ...
    @property
    def is_gpu_mode(self) -> bool:
        """ハードウェアアクセラレーションがGPUモードかどうか。"""
        ...
    @property
    def metas(self) -> List[SpeakerMeta]:
        """メタ情報。"""
        ...
    def load_voice_model(self, model: VoiceModel) -> None:
        """
        モデルを読み込む。

        Parameters
        ----------
        style_id
            読み込むモデルのスタイルID。
        """
        ...
    def unload_voice_model(self, voice_model_id: Union[VoiceModelId, UUID]) -> None:
        """
        音声モデルの読み込みを解除する。

        Parameters
        ----------
        voice_model_id
            音声モデルID。
        """
        ...
    def is_loaded_voice_model(self, voice_model_id: Union[VoiceModelId, UUID]) -> bool:
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
    def audio_query_from_kana(
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
    def audio_query(
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
    def create_accent_phrases_from_kana(
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
    def create_accent_phrases(
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
    def replace_mora_data(
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
    def replace_phoneme_length(
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
    def replace_mora_pitch(
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
    def synthesis(
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
    def tts_from_kana(
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
    def tts(
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
    def load(self, path: str) -> None:
        """ファイルに保存されたユーザー辞書を読み込む。

        Parameters
        ----------
        path
            ユーザー辞書のパス。
        """
        ...
    def save(self, path: str) -> None:
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
