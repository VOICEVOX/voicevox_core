"""
ユーザー辞書の単語が反映されるかをテストする。

``test_pseudo_raii_for_asyncio_synthesizer`` と対になる。
"""

# AudioQueryのkanaを比較して変化するかどうかで判断する。

from uuid import UUID

import conftest
import voicevox_core


def test_user_dict_load() -> None:
    onnxruntime = voicevox_core.blocking.Onnxruntime.load_once(
        filename=conftest.onnxruntime_filename
    )
    open_jtalk = voicevox_core.blocking.OpenJtalk(conftest.open_jtalk_dic_dir)
    model = voicevox_core.blocking.VoiceModelFile.open(conftest.model_dir)
    synthesizer = voicevox_core.blocking.Synthesizer(onnxruntime, open_jtalk)

    synthesizer.load_voice_model(model)

    audio_query_without_dict = synthesizer.create_audio_query(
        "this_word_should_not_exist_in_default_dictionary", style_id=0
    )

    temp_dict = voicevox_core.blocking.UserDict()
    uuid = temp_dict.add_word(
        voicevox_core.UserDictWord(
            surface="this_word_should_not_exist_in_default_dictionary",
            pronunciation="アイウエオ",
            accent_type=0,
        )
    )
    assert isinstance(uuid, UUID)

    open_jtalk.use_user_dict(temp_dict)

    audio_query_with_dict = synthesizer.create_audio_query(
        "this_word_should_not_exist_in_default_dictionary", style_id=0
    )
    assert audio_query_without_dict != audio_query_with_dict
