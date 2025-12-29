from typing import TYPE_CHECKING, NoReturn

if TYPE_CHECKING:
    from voicevox_core import (
        AccentPhrase,
        AudioQuery,
        FrameAudioQuery,
        FramePhoneme,
        Mora,
        Note,
        Score,
        UserDictWord,
    )

__version__: str

class NotLoadedOpenjtalkDictError(Exception):
    """open_jtalk辞書ファイルが読み込まれていない。"""

    ...

class GpuSupportError(Exception):
    """GPUモードがサポートされていない。"""

    ...

class InitInferenceRuntimeError(Exception):
    """推論ライブラリのロードまたは初期化ができなかった。"""

    ...

class OpenZipFileError(Exception):
    """ZIPファイルを開くことに失敗した。"""

    ...

class ReadZipEntryError(Exception):
    """ZIP内のファイルが読めなかった。"""

    ...

class InvalidModelFormatError(Exception):
    """モデルの形式が不正。"""

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

class RunModelError(Exception):
    """推論に失敗した、もしくは推論結果が異常。"""

    ...

class AnalyzeTextError(Exception):
    """入力テキストの解析に失敗した。"""

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

class InvalidQueryError(ValueError):
    """AudioQuery、もしくはその一部が不正。"""

    ...

class IncompatibleQueriesError(ValueError):
    """FrameAudioQueryとScoreの組み合わせが不正。"""

    ...

class _ReservedFields:
    def __new__(cls, *args: object, **kwargs: object) -> NoReturn: ...

def _audio_query_from_accent_phrases(
    accent_phrases: list[AccentPhrase],
) -> AudioQuery: ...
def _audio_query_from_json(json: str) -> AudioQuery: ...
def _audio_query_to_json(audio_query: AudioQuery) -> str: ...
def _validate_mora(mora: Mora) -> None: ...
def _validate_accent_phrase(accent_phrase: AccentPhrase) -> None: ...
def _validate_audio_query(audio_query: AudioQuery) -> None: ...
def _validate_user_dict_word(word: UserDictWord) -> None: ...
def _validate_note(note: Note) -> None: ...
def _validate_score(score: Score) -> None: ...
def _validate_frame_phoneme(frame_phoneme: FramePhoneme) -> None: ...
def _validate_frame_audio_query(frame_audio_query: FrameAudioQuery) -> None: ...
def _to_zenkaku(text: str) -> str: ...
def wav_from_s16le(pcm: bytes, sampling_rate: int, is_stereo: bool) -> bytes:
    """
    16bit PCMにヘッダを付加しWAVフォーマットのバイナリを生成する。

    Parameters
    ----------
    pcm : bytes
        16bit PCMで表現された音声データ
    sampling_rate: int
        入力pcmのサンプリングレート
    is_stereo: bool
        入力pcmがステレオかどうか

    Returns
    -------
    bytes
        WAVフォーマットで表現された音声データ
    """
    ...

def ensure_compatible(score: Score, frame_audio_query: FrameAudioQuery) -> None:
    """
    与えられた |ensure-compatible-score|_ と |ensure-compatible-frame-audio-query|_ の組み合わせが、基本周波数と音量の生成に利用できるかどうかを確認する。

    次のうちどれかを満たすなら
    |ensure-compatible-incompatible-queries-error|_ を送出する。

    - ``score`` が |ensure-compatible-score-validate|_ 。
    - ``frame_audio_query`` が |ensure-compatible-score-validate|_ 。
    - ``score`` が表す音素ID列と、 ``frame_audio_query.phonemes``
      が表す音素ID列が等しくない。ただし異なる音素の表現が同一のIDを表すことがある。

    .. |ensure-compatible-score| replace:: 楽譜
    .. _ensure-compatible-score: #voicevox_core.Score
    .. |ensure-compatible-frame-audio-query| replace:: 歌唱合成用のクエリ
    .. _ensure-compatible-frame-audio-query: #voicevox_core.FrameAudioQuery
    .. |ensure-compatible-incompatible-queries-error| replace:: ``IncompatibleQueriesError``
    .. _ensure-compatible-incompatible-queries-error: #voicevox_core.IncompatibleQueriesError
    .. |ensure-compatible-score-validate| replace:: 不正
    .. _ensure-compatible-score-validate: #voicevox_core.Score.validate
    .. |ensure-compatible-frame-audio-query-validate| replace:: 不正
    .. _ensure-compatible-frame-audio-query-validate: #voicevox_core.FrameAudioQuery.validate
    """
    ...
