import dataclasses
from typing import Literal, NewType, TypeAlias
from uuid import UUID

import pydantic

from ._rust import _to_zenkaku, _validate_pronunciation

StyleId = NewType("StyleId", int)
"""
スタイルID。

Parameters
----------
x : int
"""

CharacterVersion = NewType("CharacterVersion", str)
"""
**キャラクター**のバージョン。

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

StyleType: TypeAlias = Literal["talk", "singing_teacher", "frame_decode", "sing"]
"""

**スタイル** (_style_)に対応するモデルの種類。

===================== ==================================================
値                    説明
``"talk"``            音声合成クエリの作成と音声合成が可能。
``"singing_teacher"`` 歌唱音声合成用のクエリの作成が可能。
``"frame_decode"``    歌唱音声合成用のクエリの作成が可能。
``"sing"``            歌唱音声合成用のクエリの作成と歌唱音声合成が可能。
===================== ==================================================
"""


@pydantic.dataclasses.dataclass
class StyleMeta:
    """**スタイル** (_style_)のメタ情報。"""

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
    """**キャラクター**のメタ情報。"""

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


AccelerationMode: TypeAlias = Literal["AUTO", "CPU", "GPU"]
"""
ハードウェアアクセラレーションモードを設定する設定値。

========== ======================================================================
値         説明
``"AUTO"`` 実行環境に合った適切なハードウェアアクセラレーションモードを選択する。
``"CPU"``  ハードウェアアクセラレーションモードを"CPU"に設定する。
``"GPU"``  ハードウェアアクセラレーションモードを"GPU"に設定する。
========== ======================================================================
"""


@pydantic.dataclasses.dataclass
class Mora:
    """モーラ（子音＋母音）ごとの情報。"""

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
    """AccentPhrase (アクセント句ごとの情報)。"""

    moras: list[Mora]
    """モーラの配列。"""

    accent: int
    """アクセント箇所。"""

    pause_mora: Mora | None = None
    """後ろに無音を付けるかどうか。"""

    is_interrogative: bool = False
    """疑問系かどうか。"""


@pydantic.dataclasses.dataclass
class AudioQuery:
    """AudioQuery (音声合成用のクエリ)。"""

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

    pause_length: None = None
    """句読点などの無音時間。 ``None`` のときは無視される。デフォルト値は ``None`` 。"""

    pause_length_scale: float = 1.0
    """読点などの無音時間（倍率）。デフォルト値は ``1.0`` 。"""

    kana: str | None = None
    """
    [読み取り専用] AquesTalk風記法。

    :func:`Synthesizer.create_audio_query` が返すもののみ ``str`` となる。入力として
    のAudioQueryでは無視される。
    """


UserDictWordType: TypeAlias = Literal[
    "PROPER_NOUN", "COMMON_NOUN", "VERB", "ADJECTIVE", "SUFFIX"
]
"""
ユーザー辞書の単語の品詞。

================= ==========
値                説明
``"PROPER_NOUN"`` 固有名詞。
``"COMMON_NOUN"`` 一般名詞。
``"VERB"``        動詞。
``"ADJECTIVE"``   形容詞。
``"SUFFIX"``      語尾。
================= ==========
"""


@pydantic.dataclasses.dataclass
class UserDictWord:
    """ユーザー辞書の単語。"""

    surface: str
    """言葉の表層形。"""

    pronunciation: str
    """
    言葉の発音。

    カタカナで表記する。
    """

    accent_type: int = dataclasses.field(default=0)
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

    @pydantic.field_validator("pronunciation")
    @classmethod
    def _validate_pronunciation(cls, v):
        _validate_pronunciation(v)
        return v

    @pydantic.field_validator("surface")
    @classmethod
    def _validate_surface(cls, v):
        return _to_zenkaku(v)
