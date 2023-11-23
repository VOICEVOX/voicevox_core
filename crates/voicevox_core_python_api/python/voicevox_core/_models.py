import dataclasses
from enum import Enum
from typing import List, NewType, Optional

import pydantic

from ._rust import _to_zenkaku, _validate_pronunciation

StyleId = NewType("StyleId", int)
"""
スタイルID。

Parameters
----------
x : int
"""

StyleVersion = NewType("StyleVersion", str)
"""
スタイルのバージョン。

Parameters
----------
x : str
"""

VoiceModelId = NewType("VoiceModelId", str)
"""
音声モデルID。

Parameters
----------
x : str
"""


@pydantic.dataclasses.dataclass
class StyleMeta:
    """**スタイル** (_style_)のメタ情報。"""

    name: str
    """スタイル名。"""

    id: StyleId
    """スタイルID。"""


@pydantic.dataclasses.dataclass
class SpeakerMeta:
    """**話者** (*speaker*)のメタ情報。"""

    name: str
    """話者名。"""

    styles: List[StyleMeta]
    """話者に属するスタイル。"""

    speaker_uuid: str
    """話者のバージョン。"""

    version: StyleVersion
    """話者のUUID。"""


@pydantic.dataclasses.dataclass
class SupportedDevices:
    """
    このライブラリで利用可能なデバイスの情報。

    あくまで本ライブラリが対応しているデバイスの情報であることに注意。GPUが使える環境ではなかったとしても
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


class AccelerationMode(str, Enum):
    """
    ハードウェアアクセラレーションモードを設定する設定値。
    """

    AUTO = "AUTO"
    """
    実行環境に合った適切なハードウェアアクセラレーションモードを選択する。
    """

    CPU = "CPU"
    """ハードウェアアクセラレーションモードを"CPU"に設定する。"""

    GPU = "GPU"
    """ハードウェアアクセラレーションモードを"GPU"に設定する。"""


@pydantic.dataclasses.dataclass
class Mora:
    """モーラ（子音＋母音）ごとの情報。"""

    text: str
    """文字。"""

    consonant: Optional[str]
    """子音の音素。"""

    consonant_length: Optional[float]
    """子音の音長。"""

    vowel: str
    """母音の音素。"""

    vowel_length: float
    """母音の音長。"""

    pitch: float
    """音高。"""


@pydantic.dataclasses.dataclass
class AccentPhrase:
    """AccentPhrase (アクセント句ごとの情報)。"""

    moras: List[Mora]
    """モーラの配列。"""

    accent: int
    """アクセント箇所。"""

    pause_mora: Optional[Mora]
    """後ろに無音を付けるかどうか。"""

    is_interrogative: bool
    """疑問系かどうか。"""


@pydantic.dataclasses.dataclass
class AudioQuery:
    """AudioQuery (音声合成用のクエリ)。"""

    accent_phrases: List[AccentPhrase]
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

    kana: Optional[str]
    """
    [読み取り専用] AquesTalk風記法。

    :func:`Synthesizer.audio_query` が返すもののみ ``str`` となる。入力としてのAudioQueryでは無視さ
    れる。
    """


class UserDictWordType(str, Enum):
    """ユーザー辞書の単語の品詞。"""

    PROPER_NOUN = "PROPER_NOUN"
    """固有名詞。"""

    COMMON_NOUN = "COMMON_NOUN"
    """一般名詞。"""

    VERB = "VERB"
    """動詞。"""

    ADJECTIVE = "ADJECTIVE"
    """形容詞。"""

    SUFFIX = "SUFFIX"
    """語尾。"""


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

    word_type: UserDictWordType = dataclasses.field(
        default=UserDictWordType.COMMON_NOUN
    )
    """品詞。"""

    priority: int = dataclasses.field(default=5)
    """
    単語の優先度。

    0から10までの整数。
    数字が大きいほど優先度が高くなる。
    1から9までの値を指定することを推奨する。
    """

    @pydantic.validator("pronunciation")
    def _validate_pronunciation(cls, v):
        _validate_pronunciation(v)
        return v

    @pydantic.validator("surface")
    def _validate_surface(cls, v):
        return _to_zenkaku(v)
