from enum import Enum
from typing import List, Optional

import pydantic


@pydantic.dataclasses.dataclass
class StyleMeta:
    """スタイルのメタ情報。"""

    name: str
    """スタイル名。"""

    id: int
    """スタイルID。"""


@pydantic.dataclasses.dataclass
class SpeakerMeta:
    """話者のメタ情報。"""

    name: str
    """話者名。"""

    styles: List[StyleMeta]
    """話者に属するスタイル。"""

    speaker_uuid: str
    """話者のバージョン。"""

    version: str
    """話者のUUID。"""


@pydantic.dataclasses.dataclass
class SupportedDevices:
    """このライブラリで利用可能なデバイスの情報。"""

    cpu: bool
    """
    CPUが利用可能。

    常に ``true`` 。
    """

    cuda: bool
    """
    `CUDA Execution Provider <https://onnxruntime.ai/docs/execution-providers/CUDA-ExecutionProvider.html>`_
    (``CUDAExecutionProvider``)が利用可能。
    """

    dml: bool
    """
    `DirectML Execution Provider <https://onnxruntime.ai/docs/execution-providers/DirectML-ExecutionProvider.html>`_
    (``DmlExecutionProvider``)が利用可能。
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
    """モーラの列。"""

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
    """アクセント句の列。"""

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
    [読み取り専用] AquesTalkライクな読み仮名。

    :func:`Synthesizer.audio_query` が返すもののみ ``str`` となる。入力としてのAudioQueryでは無視さ
    れる。
    """
