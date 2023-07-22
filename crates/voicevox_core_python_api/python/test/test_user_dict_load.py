# ユーザー辞書の単語が反映されるかをテストする。
# AudioQueryのkanaを比較して変化するかどうかで判断する。

from uuid import UUID
import pytest
import conftest  # noqa: F401
import voicevox_core  # noqa: F401


@pytest.mark.asyncio
async def test_user_dict_load() -> None:
    open_jtalk = voicevox_core.OpenJtalk(conftest.open_jtalk_dic_dir)
    model = await voicevox_core.VoiceModel.from_path(conftest.model_dir)
    synthesizer = await voicevox_core.Synthesizer.new_with_initialize(
        open_jtalk=open_jtalk,
    )

    await synthesizer.load_voice_model(model)

    audio_query_without_dict = await synthesizer.audio_query(
        "this_word_should_not_exist_in_default_dictionary", style_id=0, kana=False
    )

    temp_dict = voicevox_core.UserDict()
    uuid = temp_dict.add_word(
        voicevox_core.UserDictWord(
            surface="this_word_should_not_exist_in_default_dictionary",
            pronunciation="アイウエオ",
        )
    )
    assert isinstance(uuid, UUID)

    open_jtalk.use_user_dict(temp_dict)

    audio_query_with_dict = await synthesizer.audio_query(
        "this_word_should_not_exist_in_default_dictionary", style_id=0, kana=False
    )
    assert audio_query_without_dict != audio_query_with_dict
