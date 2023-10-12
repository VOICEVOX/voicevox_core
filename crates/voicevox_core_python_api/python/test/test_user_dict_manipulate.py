# ユーザー辞書の操作をテストする。
# どのコードがどの操作を行っているかはコメントを参照。

import os
import tempfile
from uuid import UUID

import pydantic
import pytest
import voicevox_core  # noqa: F401


@pytest.mark.asyncio
async def test_user_dict_load() -> None:
    dict_a = voicevox_core.UserDict()

    # 単語の追加
    uuid_a = dict_a.add_word(
        voicevox_core.UserDictWord(
            surface="hoge",
            pronunciation="ホゲ",
        )
    )
    assert isinstance(uuid_a, UUID)
    assert dict_a.words[uuid_a].surface == "ｈｏｇｅ"
    assert dict_a.words[uuid_a].pronunciation == "ホゲ"

    # 単語の更新
    dict_a.update_word(
        uuid_a,
        voicevox_core.UserDictWord(
            surface="fuga",
            pronunciation="フガ",
        ),
    )

    assert dict_a.words[uuid_a].surface == "ｆｕｇａ"
    assert dict_a.words[uuid_a].pronunciation == "フガ"

    # ユーザー辞書のインポート
    dict_b = voicevox_core.UserDict()
    uuid_b = dict_b.add_word(
        voicevox_core.UserDictWord(
            surface="foo",
            pronunciation="フー",
        )
    )

    dict_a.import_dict(dict_b)
    assert uuid_b in dict_a.words

    # ユーザー辞書のエクスポート
    dict_c = voicevox_core.UserDict()
    uuid_c = dict_c.add_word(
        voicevox_core.UserDictWord(
            surface="bar",
            pronunciation="バー",
        )
    )
    temp_path_fd, temp_path = tempfile.mkstemp()
    os.close(temp_path_fd)
    dict_c.save(temp_path)
    dict_a.load(temp_path)
    assert uuid_a in dict_a.words
    assert uuid_c in dict_a.words

    # 単語の削除
    dict_a.remove_word(uuid_a)
    assert uuid_a not in dict_a.words
    assert uuid_c in dict_a.words

    # 単語のバリデーション
    with pytest.raises(pydantic.ValidationError):
        dict_a.add_word(
            voicevox_core.UserDictWord(
                surface="",
                pronunciation="カタカナ以外の文字",
            )
        )
