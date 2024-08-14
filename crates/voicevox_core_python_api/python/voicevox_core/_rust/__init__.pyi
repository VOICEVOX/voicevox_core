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

class MlInferenceError(Exception):
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
