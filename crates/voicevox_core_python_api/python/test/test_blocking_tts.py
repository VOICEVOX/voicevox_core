"""
音声合成を行う。

``test_asyncio_tts`` と対になる。
"""

import conftest
import pytest
from voicevox_core import AudioQuery
from voicevox_core.blocking import Onnxruntime, OpenJtalk, Synthesizer, VoiceModelFile


def test(synthesizer: Synthesizer) -> None:
    TEXT = "こんにちは？"
    STYLE_ID = 0

    wav1 = synthesizer.tts(TEXT, STYLE_ID)

    query = synthesizer.create_audio_query(TEXT, STYLE_ID)
    wav2 = synthesizer.synthesis(query, STYLE_ID)

    phrases = synthesizer.create_accent_phrases(TEXT, STYLE_ID)
    query = AudioQuery.from_accent_phrases(phrases)
    wav3 = synthesizer.synthesis(query, STYLE_ID)

    phrases = synthesizer.open_jtalk.analyze(TEXT)
    phrases = synthesizer.replace_mora_data(phrases, STYLE_ID)
    query = AudioQuery.from_accent_phrases(phrases)
    wav4 = synthesizer.synthesis(query, STYLE_ID)

    phrases = synthesizer.open_jtalk.analyze(TEXT)
    phrases = synthesizer.replace_phoneme_length(phrases, STYLE_ID)
    phrases = synthesizer.replace_mora_pitch(phrases, STYLE_ID)
    query = AudioQuery.from_accent_phrases(phrases)
    wav5 = synthesizer.synthesis(query, STYLE_ID)

    wav6 = synthesizer.tts(TEXT, STYLE_ID, enable_interrogative_upspeak=False)

    query = synthesizer.create_audio_query(TEXT, STYLE_ID)
    wav7 = synthesizer.synthesis(query, STYLE_ID, enable_interrogative_upspeak=False)

    phrases = synthesizer.create_accent_phrases(TEXT, STYLE_ID)
    query = AudioQuery.from_accent_phrases(phrases)
    wav8 = synthesizer.synthesis(query, STYLE_ID, enable_interrogative_upspeak=False)

    phrases = synthesizer.open_jtalk.analyze(TEXT)
    phrases = synthesizer.replace_mora_data(phrases, STYLE_ID)
    query = AudioQuery.from_accent_phrases(phrases)
    wav9 = synthesizer.synthesis(query, STYLE_ID, enable_interrogative_upspeak=False)

    phrases = synthesizer.open_jtalk.analyze(TEXT)
    phrases = synthesizer.replace_phoneme_length(phrases, STYLE_ID)
    phrases = synthesizer.replace_mora_pitch(phrases, STYLE_ID)
    query = AudioQuery.from_accent_phrases(phrases)
    wav10 = synthesizer.synthesis(query, STYLE_ID, enable_interrogative_upspeak=False)

    assert wav1 != wav6
    assert len({wav1, wav2, wav3, wav4, wav5}) == 1
    assert len({wav6, wav7, wav8, wav9, wav10}) == 1


@pytest.fixture
def synthesizer() -> Synthesizer:
    onnxruntime = Onnxruntime.load_once(filename=conftest.onnxruntime_filename)
    open_jtalk = OpenJtalk(conftest.open_jtalk_dic_dir)
    synthesizer = Synthesizer(onnxruntime, open_jtalk, acceleration_mode="CPU")
    with VoiceModelFile.open(conftest.model_dir) as model:
        synthesizer.load_voice_model(model)
    return synthesizer
