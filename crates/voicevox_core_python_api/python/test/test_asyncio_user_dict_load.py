"""
ユーザー辞書の単語が反映されるかをテストする。

``test_pseudo_raii_for_blocking_synthesizer`` と対になる。
"""

# AudioQueryのkanaを比較して変化するかどうかで判断する。

import multiprocessing
from uuid import UUID

import conftest  # noqa: F401
import pytest
import voicevox_core  # noqa: F401


@pytest.mark.asyncio
async def test_user_dict_load() -> None:
    onnxruntime = await voicevox_core.asyncio.Onnxruntime.load_once(
        filename=conftest.onnxruntime_filename
    )
    open_jtalk = await voicevox_core.asyncio.OpenJtalk.new(conftest.open_jtalk_dic_dir)
    model = await voicevox_core.asyncio.VoiceModelFile.open(conftest.model_dir)
    synthesizer = voicevox_core.asyncio.Synthesizer(
        onnxruntime,
        open_jtalk,
        cpu_num_threads=max(
            multiprocessing.cpu_count(), 2
        ),  # https://github.com/VOICEVOX/voicevox_core/issues/888
    )

    await synthesizer.load_voice_model(model)

    audio_query_without_dict = await synthesizer.create_audio_query(
        "this_word_should_not_exist_in_default_dictionary", style_id=0
    )

    temp_dict = voicevox_core.asyncio.UserDict()
    uuid = temp_dict.add_word(
        voicevox_core.UserDictWord(
            surface="this_word_should_not_exist_in_default_dictionary",
            pronunciation="アイウエオ",
            accent_type=0,
        )
    )
    assert isinstance(uuid, UUID)

    await open_jtalk.use_user_dict(temp_dict)

    audio_query_with_dict = await synthesizer.create_audio_query(
        "this_word_should_not_exist_in_default_dictionary", style_id=0
    )
    assert audio_query_without_dict != audio_query_with_dict
