from os import PathLike
from typing import TYPE_CHECKING, NoReturn, Union
from uuid import UUID

if TYPE_CHECKING:
    from voicevox_core import (
        AccelerationMode,
        AccentPhrase,
        AudioQuery,
        CharacterMeta,
        StyleId,
        SupportedDevices,
        UserDictWord,
        VoiceModelId,
    )

class VoiceModelFile:
    """
    音声モデルファイル。"""

    def __new__(
        cls, *args: tuple[object], **kwargs: dict[object, object]
    ) -> NoReturn: ...
    @staticmethod
    async def open(path: str | PathLike[str]) -> VoiceModelFile:
        """
        VVMファイルを開く。

        Parameters
        ----------
        path
            VVMファイルへのパス。
        """
        ...
    async def close(self) -> None:
        """
        VVMファイルを閉じる。

        このメソッドが呼ばれた段階で :attr:`Synthesizer.load_voice_model`
        からのアクセスが継続中の場合、アクセスが終わるまで待つ。
        """
        ...
    @property
    def id(self) -> VoiceModelId:
        """
        ID。

        :attr:`close` および :attr:`__aexit__` の後でも利用可能。
        """
        ...
    @property
    def metas(self) -> list[CharacterMeta]:
        """
        メタ情報。

        この中身を書き換えても、 ``VoiceModelFile`` としての動作には影響しない。

        :attr:`close` および :attr:`__aexit__` の後でも利用可能。
        """
        ...
    async def __aenter__(self) -> "VoiceModelFile": ...
    async def __aexit__(self, exc_type, exc_value, traceback) -> None: ...

class Onnxruntime:
    """
    ONNX Runtime。

    シングルトンであり、インスタンスは高々一つ。

    .. code-block::

        ort1 = await Onnxruntime.load_once()
        ort2 = Onnxruntime.get()
        assert ort2
        assert ort2 is ort1

    .. code-block::

        ort = await voicevox_core.asyncio.Onnxruntime.load_once()
        assert voicevox_core.blocking.Onnxruntime.get()
    """

    # ここの定数値が本物と合致するかどうかは、test_type_stub_consts.pyで担保する。

    LIB_NAME: str = "voicevox_onnxruntime"
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

    def __new__(
        cls, *args: tuple[object], **kwargs: dict[object, object]
    ) -> NoReturn: ...
    @staticmethod
    def get() -> Union["Onnxruntime", None]:
        """
        インスタンスが既に作られているならそれを得る。

        作られていなければ ``None`` を返す。
        """
        ...
    @staticmethod
    async def load_once(*, filename: str = LIB_VERSIONED_FILENAME) -> "Onnxruntime":
        """
        ONNX Runtimeをロードして初期化する。

        一度成功したら、以後は引数を無視して同じインスタンスを返す。

        Parameters
        ----------
        filename
            ONNX Runtimeのファイル名（モジュール名）もしくはファイルパス。
            ``dlopen``/`LoadLibraryExW
            <https://learn.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-loadlibraryexw>`_
            の引数に使われる。
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
    """

    def __new__(
        cls, *args: tuple[object], **kwargs: dict[object, object]
    ) -> NoReturn: ...
    @staticmethod
    async def new(open_jtalk_dict_dir: str | PathLike[str]) -> "OpenJtalk":
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
    async def analyze(self, text: str) -> list[AccentPhrase]:
        """
        日本語のテキストを解析する。

        Parameters
        ----------
        text
            日本語のテキスト。
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
        CPU利用数を指定。0を指定すると環境に合わせたCPUが利用される。未調査ではあるが、物理コアの数+1とするのが適切な可能性がある
        (`VOICEVOX/voicevox_core#902 <https://github.com/VOICEVOX/voicevox_core/issues/902>`_)。
    """

    def __init__(
        self,
        onnxruntime: Onnxruntime,
        open_jtalk: OpenJtalk,
        *,
        acceleration_mode: AccelerationMode = "AUTO",
        cpu_num_threads: int = 0,
    ) -> None: ...
    def __repr__(self) -> str: ...
    async def __aenter__(self) -> "Synthesizer": ...
    async def __aexit__(self, exc_type, exc_value, traceback) -> None: ...
    @property
    def onnxruntime(self) -> Onnxruntime:
        """ONNX Runtime。"""
        ...
    @property
    def open_jtalk(self) -> OpenJtalk:
        """Open JTalk。"""
        ...
    @property
    def is_gpu_mode(self) -> bool:
        """ハードウェアアクセラレーションがGPUモードかどうか。"""
        ...
    def metas(self) -> list[CharacterMeta]:
        """メタ情報。"""
        ...
    async def load_voice_model(self, model: VoiceModelFile) -> None:
        """
        モデルを読み込む。

        Parameters
        ----------
        style_id
            読み込むモデルのスタイルID。
        """
        ...
    def unload_voice_model(self, voice_model_id: VoiceModelId | UUID) -> None:
        """
        音声モデルの読み込みを解除する。

        Parameters
        ----------
        voice_model_id
            音声モデルID。
        """
        ...
    def is_loaded_voice_model(self, voice_model_id: VoiceModelId | UUID) -> bool:
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
    async def create_audio_query_from_kana(
        self,
        kana: str,
        style_id: StyleId | int,
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
        スタイルとテキストから生成された :class:`AudioQuery` 。
        """
        ...
    async def create_audio_query(
        self,
        text: str,
        style_id: StyleId | int,
    ) -> AudioQuery:
        """
        日本語のテキストから :class:`AudioQuery` を生成する。

        :func:`create_accent_phrases` と |from-accent-phrases|_
        が一体になったショートハンド。詳細は `テキスト音声合成の流れ
        <https://github.com/VOICEVOX/voicevox_core/blob/main/docs/guide/user/tts-process.md>`_
        を参照。

        .. |from-accent-phrases| replace:: ``AudioQuery.from_accent_phrases()``
        .. _from-accent-phrases: ../index.html#voicevox_core.AudioQuery.from_accent_phrases

        Parameters
        ----------
        text
            UTF-8の日本語テキスト。
        style_id
            スタイルID。

        Returns
        -------
        スタイルとテキストから生成された :class:`AudioQuery` 。
        """
        ...
    async def create_accent_phrases_from_kana(
        self,
        kana: str,
        style_id: StyleId | int,
    ) -> list[AccentPhrase]:
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
        style_id: StyleId | int,
    ) -> list[AccentPhrase]:
        """
        日本語のテキストからAccentPhrase（アクセント句）の配列を生成する。

        :func:`OpenJtalk.analyze` と :func:`replace_mora_data`
        が一体になったショートハンド。詳細は `テキスト音声合成の流れ
        <https://github.com/VOICEVOX/voicevox_core/blob/main/docs/guide/user/tts-process.md>`_
        を参照。

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
        accent_phrases: list[AccentPhrase],
        style_id: StyleId | int,
    ) -> list[AccentPhrase]:
        """
        アクセント句の音高・音素長を変更した新しいアクセント句の配列を生成する。

        元のアクセント句の音高・音素長は変更されない。

        :func:`replace_phoneme_length` と :func:`replace_mora_pitch`
        が一体になったショートハンド。詳細は `テキスト音声合成の流れ
        <https://github.com/VOICEVOX/voicevox_core/blob/main/docs/guide/user/tts-process.md>`_
        を参照。

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
        accent_phrases: list[AccentPhrase],
        style_id: StyleId | int,
    ) -> list[AccentPhrase]:
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
        accent_phrases: list[AccentPhrase],
        style_id: StyleId | int,
    ) -> list[AccentPhrase]:
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
        style_id: StyleId | int,
        *,
        enable_interrogative_upspeak: bool = True,
        cancellable: bool = False,
    ) -> bytes:
        """
        :class:`AudioQuery` から音声合成する。

        ``cancellable``
        を有効化しない限り、非同期タスクとしてキャンセルしても終わるまで停止しない。

        Parameters
        ----------
        audio_query
            :class:`AudioQuery` 。
        style_id
            スタイルID。
        enable_interrogative_upspeak
            疑問文の調整を有効にするかどうか。
        cancellable
            音声モデルの実行をキャンセル可能にするかどうか。このオプションを有効にすると、負荷がかかっている状況下でハングする可能性がある。そのためデフォルトでは無効化されている。
            `VOICEVOX/voicevox_core#968 <https://github.com/VOICEVOX/voicevox_core/issues/968>`_ を参照。

        Returns
        -------
        WAVデータ。
        """
        ...
    async def tts_from_kana(
        self,
        kana: str,
        style_id: StyleId | int,
        *,
        enable_interrogative_upspeak: bool = True,
        cancellable: bool = False,
    ) -> bytes:
        """
        AquesTalk風記法から音声合成を行う。

        ``cancellable``
        を有効化しない限り、非同期タスクとしてキャンセルしても終わるまで停止しない。

        Parameters
        ----------
        kana
            AquesTalk風記法。
        style_id
            スタイルID。
        enable_interrogative_upspeak
            疑問文の調整を有効にするかどうか。
        cancellable
            音声モデルの実行をキャンセル可能にするかどうか。このオプションを有効にすると、負荷がかかっている状況下でハングする可能性がある。そのためデフォルトでは無効化されている。
            `VOICEVOX/voicevox_core#968 <https://github.com/VOICEVOX/voicevox_core/issues/968>`_ を参照。
        """
        ...
    async def tts(
        self,
        text: str,
        style_id: StyleId | int,
        *,
        enable_interrogative_upspeak: bool = True,
        cancellable: bool = False,
    ) -> bytes:
        """
        日本語のテキストから音声合成を行う。

        :func:`create_audio_query` と :func:`synthesis`
        が一体になったショートハンド。詳細は `テキスト音声合成の流れ
        <https://github.com/VOICEVOX/voicevox_core/blob/main/docs/guide/user/tts-process.md>`_
        を参照。

        ``cancellable``
        を有効化しない限り、非同期タスクとしてキャンセルしても終わるまで停止しない。

        Parameters
        ----------
        text
            UTF-8の日本語テキスト。
        style_id
            スタイルID。
        enable_interrogative_upspeak
            疑問文の調整を有効にするかどうか。
        cancellable
            音声モデルの実行をキャンセル可能にするかどうか。このオプションを有効にすると、負荷がかかっている状況下でハングする可能性がある。そのためデフォルトでは無効化されている。
            `VOICEVOX/voicevox_core#968 <https://github.com/VOICEVOX/voicevox_core/issues/968>`_ を参照。

        Returns
        -------
        WAVデータ。
        """
        ...
    async def close(self) -> None: ...

class UserDict:
    """ユーザー辞書。"""

    def to_dict(self) -> dict[UUID, UserDictWord]:
        """このオプジェクトを :class:`dict` に変換する。"""
        ...
    def __init__(self) -> None: ...
    async def load(self, path: str | PathLike[str]) -> None:
        """ファイルに保存されたユーザー辞書を読み込む。

        Parameters
        ----------
        path
            ユーザー辞書のパス。
        """
        ...
    async def save(self, path: str | PathLike[str]) -> None:
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
