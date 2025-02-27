import dataclasses
from typing import Literal, NewType, TypeAlias
from uuid import UUID

import pydantic.alias_generators
from pydantic import ConfigDict
from pydantic_core import ArgsKwargs

from .._rust import (
    _audio_query_from_accent_phrases,
    _to_zenkaku,
    _validate_user_dict_word,
)
from ._please_do_not_use import _Reserved

StyleId = NewType("StyleId", int)
"""
スタイルID。

VOICEVOXにおける、ある |styleid-character|_ のある |styleid-style|_ を指す。

.. |styleid-character| replace:: *キャラクター*
.. _styleid-character: #voicevox_core.CharacterMeta
.. |styleid-style| replace:: *スタイル*
.. _styleid-style: #voicevox_core.StyleMeta

Parameters
----------
x : int
"""

CharacterVersion = NewType("CharacterVersion", str)
"""
|characterversion-character|_ のバージョン。

.. |characterversion-character| replace:: *キャラクター*
.. _characterversion-character: #voicevox_core.CharacterMeta

Parameters
----------
x : str
"""

VoiceModelId = NewType("VoiceModelId", UUID)
"""
音声モデルID。

Parameters
----------
x : UUID
"""

StyleType: TypeAlias = (
    Literal["talk", "singing_teacher", "frame_decode", "sing"] | _Reserved
)
"""

|styletype-style|_ に対応するモデルの種類。

.. |styletype-style| replace:: *スタイル*
.. _styletype-style: #voicevox_core.StyleMeta

===================== ==================================================
値                    説明
``"talk"``            音声合成クエリの作成と音声合成が可能。
``"singing_teacher"`` 歌唱音声合成用のクエリの作成が可能。
``"frame_decode"``    歌唱音声合成が可能。
``"sing"``            歌唱音声合成用のクエリの作成と歌唱音声合成が可能。
``_Reserved``         将来のために予約されている値。この値が存在することは決してない。
                      ``str`` のサブタイプであるため、 ``StyleType`` を ``str`` として
                      扱うことは可能。
===================== ==================================================

``_Reserved`` の存在により、例えば次のコードはPyright/Pylanceの型検査に通らない。これは意図的なデザインである。

.. code-block::

    def _(style_type: StyleType) -> int:
        match style_type:
            case "talk":
                return 0
            case "singing_teacher":
                return 1
            case "frame_decode":
                return 2
            case "sing":
                return 3

.. code-block:: text

    error: Function with declared return type "int" must return value on all code paths
    "None" is not assignable to "int" (reportReturnType)

``str`` として扱うことは可能。

.. code-block::

    def _(style_type: StyleType):
        _: str = style_type  # OK
"""


def _(style_type: StyleType):
    _: str = style_type


@pydantic.dataclasses.dataclass
class StyleMeta:
    """
    *スタイル* のメタ情報。

    現在は |pydantic-dataclasses-dataclass-stylemeta|_ ではあるが、将来的には
    |de-pydantic-stylemeta|_ 。

    .. |pydantic-dataclasses-dataclass-stylemeta| replace:: ``@pydantic.dataclasses.dataclass``
    .. _pydantic-dataclasses-dataclass-stylemeta: https://docs.pydantic.dev/2.10/concepts/dataclasses/
    .. |de-pydantic-stylemeta| replace:: JSONと相互変換するメソッドのみ付いた、素の ``dataclass`` になる予定
    .. _de-pydantic-stylemeta: https://github.com/VOICEVOX/voicevox_core/issues/960#issuecomment-2629424401
    """

    name: str
    """スタイル名。"""

    id: StyleId
    """スタイルID。"""

    type: StyleType = dataclasses.field(default="talk")
    """スタイルに対応するモデルの種類。"""

    order: int | None = None
    """
    スタイルの順番。

    :attr:`CharacterMeta.styles` は、この値に対して昇順に並んでいるべきである。
    """


@pydantic.dataclasses.dataclass
class CharacterMeta:
    """
    *キャラクター* のメタ情報。

    現在は |pydantic-dataclasses-dataclass-charactermeta|_ ではあるが、将来的には
    |de-pydantic-charactermeta|_ 。

    .. |pydantic-dataclasses-dataclass-charactermeta| replace:: ``@pydantic.dataclasses.dataclass``
    .. _pydantic-dataclasses-dataclass-charactermeta: https://docs.pydantic.dev/2.10/concepts/dataclasses/
    .. |de-pydantic-charactermeta| replace:: JSONと相互変換するメソッドのみ付いた、素の ``dataclass`` になる予定
    .. _de-pydantic-charactermeta: https://github.com/VOICEVOX/voicevox_core/issues/960#issuecomment-2629424401
    """

    name: str
    """キャラクター名。"""

    styles: list[StyleMeta]
    """キャラクターに属するスタイル。"""

    speaker_uuid: str
    """キャラクターのUUID。"""

    version: CharacterVersion
    """キャラクターのバージョン。"""

    order: int | None = None
    """
    キャラクターの順番。

    ``CharacterMeta`` の列は、この値に対して昇順に並んでいるべきである。
    """


@pydantic.dataclasses.dataclass
class SupportedDevices:
    """
    ONNX Runtimeとして利用可能なデバイスの情報。

    あくまでONNX Runtimeが対応しているデバイスの情報であることに注意。GPUが使える環境ではなかったとしても
    ``cuda`` や ``dml`` は ``True`` を示しうる。

    JSONからの変換も含め、VOICEVOX CORE以外が作ることはできない。作ろうとした場合
    ``TypeError`` となる。

    現在は |pydantic-dataclasses-dataclass-supporteddevices|_ ではあるが、将来的には
    |de-pydantic-supporteddevices|_ 。

    .. |pydantic-dataclasses-dataclass-supporteddevices| replace:: ``@pydantic.dataclasses.dataclass``
    .. _pydantic-dataclasses-dataclass-supporteddevices: https://docs.pydantic.dev/2.10/concepts/dataclasses/
    .. |de-pydantic-supporteddevices| replace:: JSONと相互変換するメソッドのみ付いた、素の ``dataclass`` になる予定
    .. _de-pydantic-supporteddevices: https://github.com/VOICEVOX/voicevox_core/issues/960#issuecomment-2629424401
    """

    cpu: bool
    """
    CPUが利用可能。

    常に ``True`` 。
    """

    cuda: bool
    """
    CUDAが利用可能。

    ONNX Runtimeの `CUDA Execution Provider <https://onnxruntime.ai/docs/execution-providers/CUDA-ExecutionProvider.html>`_
    (``CUDAExecutionProvider``)に対応する。必要な環境についてはそちらを参照。
    """

    dml: bool
    """
    DirectMLが利用可能。

    ONNX Runtimeの `DirectML Execution Provider <https://onnxruntime.ai/docs/execution-providers/DirectML-ExecutionProvider.html>`_
    (``DmlExecutionProvider``)に対応する。必要な環境についてはそちらを参照。
    """

    @pydantic.model_validator(mode="before")
    @staticmethod
    def _deny_unless_from_pyo3(data: ArgsKwargs) -> ArgsKwargs:
        if "I AM FROM PYO3" not in data.args:
            raise TypeError("You cannot deserialize `SupportedDevices`")
        return ArgsKwargs((), kwargs=data.kwargs)


AccelerationMode: TypeAlias = Literal["AUTO", "CPU", "GPU"] | _Reserved
"""
ハードウェアアクセラレーションモードを設定する設定値。

============= =======================================================================
値            説明
``"AUTO"``    実行環境に合った適切なハードウェアアクセラレーションモードを選択する。
``"CPU"``     ハードウェアアクセラレーションモードを"CPU"に設定する。
``"GPU"``     ハードウェアアクセラレーションモードを"GPU"に設定する。
``_Reserved`` 将来のために予約されている値。この値が存在することは決してない。
              ``str`` のサブタイプであるため、 ``AccelerationMode`` を ``str`` として
              扱うことは可能。
============= =======================================================================

``_Reserved`` の存在により、例えば次のコードはPyright/Pylanceの型検査に通らない。これは意図的なデザインである。

.. code-block::

    def _(mode: AccelerationMode) -> int:
        match mode:
            case "AUTO":
                return 0
            case "CPU":
                return 1
            case "GPU":
                return 2

.. code-block:: text

    error: Function with declared return type "int" must return value on all code paths
    "None" is not assignable to "int" (reportReturnType)

``str`` として扱うことは可能。

.. code-block::

    def _(mode: AccelerationMode):
        _: str = mode  # OK
"""


def _(mode: AccelerationMode):
    _: str = mode


@pydantic.dataclasses.dataclass
class Mora:
    """
    モーラ（子音＋母音）ごとの情報。

    現在は |pydantic-dataclasses-dataclass-mora|_ ではあるが、将来的には
    |de-pydantic-mora|_ 。

    .. |pydantic-dataclasses-dataclass-mora| replace:: ``@pydantic.dataclasses.dataclass``
    .. _pydantic-dataclasses-dataclass-mora: https://docs.pydantic.dev/2.10/concepts/dataclasses/
    .. |de-pydantic-mora| replace:: JSONと相互変換するメソッドのみ付いた、素の ``dataclass`` になる予定
    .. _de-pydantic-mora: https://github.com/VOICEVOX/voicevox_core/issues/960#issuecomment-2629424401
    """

    text: str
    """文字。"""

    vowel: str
    """母音の音素。"""

    vowel_length: float
    """母音の音長。"""

    pitch: float
    """音高。"""

    consonant: str | None = None
    """子音の音素。"""

    consonant_length: float | None = None
    """子音の音長。"""


@pydantic.dataclasses.dataclass
class AccentPhrase:
    """
    AccentPhrase (アクセント句ごとの情報)。

    現在は |pydantic-dataclasses-dataclass-accentphrase|_ ではあるが、将来的には
    |de-pydantic-accentphrase|_ 。

    .. |pydantic-dataclasses-dataclass-accentphrase| replace:: ``@pydantic.dataclasses.dataclass``
    .. _pydantic-dataclasses-dataclass-accentphrase: https://docs.pydantic.dev/2.10/concepts/dataclasses/
    .. |de-pydantic-accentphrase| replace:: JSONと相互変換するメソッドのみ付いた、素の ``dataclass`` になる予定
    .. _de-pydantic-accentphrase: https://github.com/VOICEVOX/voicevox_core/issues/960#issuecomment-2629424401
    """

    moras: list[Mora]
    """モーラの配列。"""

    accent: int
    """アクセント箇所。"""

    pause_mora: Mora | None = None
    """後ろに無音を付けるかどうか。"""

    is_interrogative: bool = False
    """疑問系かどうか。"""


def _rename_audio_query_field(name: str) -> str:
    match name:
        case "accent_phrases":
            return "accent_phrases"
        case _:
            return pydantic.alias_generators.to_camel(name)


@pydantic.dataclasses.dataclass(
    config=ConfigDict(alias_generator=_rename_audio_query_field),
)
class AudioQuery:
    """
    AudioQuery (音声合成用のクエリ)。

    現在は |pydantic-dataclasses-dataclass-audioquery|_ ではあるが、将来的には
    |de-pydantic-audioquery|_ 。

    .. |pydantic-dataclasses-dataclass-audioquery| replace:: ``@pydantic.dataclasses.dataclass``
    .. _pydantic-dataclasses-dataclass-audioquery: https://docs.pydantic.dev/2.10/concepts/dataclasses/
    .. |de-pydantic-audioquery| replace:: JSONと相互変換するメソッドのみ付いた、素の ``dataclass`` になる予定
    .. _de-pydantic-audioquery: https://github.com/VOICEVOX/voicevox_core/issues/960#issuecomment-2629424401

    JSONへの変換は ``pydantic.TypeAdapter.dump_json`` を用いなければならない。また、
    ``dump_json`` や ``json_schema`` を用いるときは ``by_alias=True``
    を指定しなければならない。

    .. code-block::

        from pydantic import TypeAdapter
        from voicevox_core import AudioQuery

        JSON = '{"accent_phrases":[],"speedScale":1.0,"pitchScale":0.0,"intonationScale":1.0,"volumeScale":1.0,"prePhonemeLength":0.1,"postPhonemeLength":0.1,"outputSamplingRate":24000,"outputStereo":false,"kana":null}'
        query = TypeAdapter(AudioQuery).validate_json(JSON)

        # `JSON`が必須フィールドを含んでいる場合のみ
        assert TypeAdapter(AudioQuery).dump_json(query, by_alias=True).decode() == JSON

        # `by_alias`が無いと、`accent_phrases`以外snake_caseのままJSONが出力されてしまう
        assert TypeAdapter(AudioQuery).dump_json(query).decode() != JSON
    """

    accent_phrases: list[AccentPhrase]
    """アクセント句の配列。"""

    speed_scale: float
    """全体の話速。"""

    pitch_scale: float
    """全体の音高。"""

    intonation_scale: float
    """全体の抑揚。"""

    volume_scale: float
    """全体の音量。"""

    pre_phoneme_length: float
    """音声の前の無音時間。"""

    post_phoneme_length: float
    """音声の後の無音時間。"""

    output_sampling_rate: int
    """音声データの出力サンプリングレート。"""

    output_stereo: bool
    """音声データをステレオ出力するか否か。"""

    kana: str | None = None
    """
    [読み取り専用] AquesTalk風記法。

    :func:`Synthesizer.create_audio_query` が返すもののみ ``str`` となる。入力として
    のAudioQueryでは無視される。
    """

    @staticmethod
    def from_accent_phrases(accent_phrases: list["AccentPhrase"]) -> "AudioQuery":
        return _audio_query_from_accent_phrases(accent_phrases)


UserDictWordType: TypeAlias = (
    Literal["PROPER_NOUN", "COMMON_NOUN", "VERB", "ADJECTIVE", "SUFFIX"] | _Reserved
)
"""
ユーザー辞書の単語の品詞。

================= ==========
値                説明
``"PROPER_NOUN"`` 固有名詞。
``"COMMON_NOUN"`` 一般名詞。
``"VERB"``        動詞。
``"ADJECTIVE"``   形容詞。
``"SUFFIX"``      語尾。
``_Reserved``     将来のために予約されている値。この値が存在することは決してない。
                  ``str`` のサブタイプであるため、 ``UserDictWordType`` を ``str`` として
                  扱うことは可能。
================= ==========

``_Reserved`` の存在により、例えば次のコードはPyright/Pylanceの型検査に通らない。これは意図的なデザインである。

.. code-block::

    def _(word_type: UserDictWordType) -> int:
        match word_type:
            case "PROPER_NOUN":
                return 0
            case "COMMON_NOUN":
                return 1
            case "VERB":
                return 2
            case "ADJECTIVE":
                return 3
            case "SUFFIX":
                return 4

.. code-block:: text

    error: Function with declared return type "int" must return value on all code paths
    "None" is not assignable to "int" (reportReturnType)

``str`` として扱うことは可能。

.. code-block::

    def _(word_type: UserDictWordType):
        _: str = word_type  # OK
"""


def _(word_type: UserDictWordType):
    _: str = word_type


@dataclasses.dataclass(frozen=True)
class UserDictWord:
    """ユーザー辞書の単語。"""

    surface: str
    """言葉の表層形。"""

    pronunciation: str
    """
    言葉の発音。

    カタカナで表記する。
    """

    accent_type: int
    """
    アクセント型。

    音が下がる場所を指す。
    """

    word_type: UserDictWordType = dataclasses.field(default="COMMON_NOUN")
    """品詞。"""

    priority: int = dataclasses.field(default=5)
    """
    単語の優先度。

    0から10までの整数。
    数字が大きいほど優先度が高くなる。
    1から9までの値を指定することを推奨する。
    """

    def __post_init__(self) -> None:
        # Pydanticが（おそらく）やっているであろう方法。
        # とりあえずPython 3.13では大丈夫らしい
        object.__setattr__(self, "surface", _to_zenkaku(self.surface))

        _validate_user_dict_word(self)
