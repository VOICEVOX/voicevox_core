from pathlib import Path
from typing import TYPE_CHECKING, Dict, Union
from uuid import UUID

if TYPE_CHECKING:
    from voicevox_core import UserDictWord

class OpenJtalk:
    """
    テキスト解析器としてのOpen JTalk。

    Parameters
    ----------
    open_jtalk_dict_dir
        Open JTalkの辞書ディレクトリ。
    """

    def __init__(self, open_jtalk_dict_dir: Union[Path, str]) -> None: ...
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
