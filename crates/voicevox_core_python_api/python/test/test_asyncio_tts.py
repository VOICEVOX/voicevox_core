"""
音声合成を行う。

``test_blocking_tts`` と対になる。
"""

import multiprocessing
import platform

import conftest
import pytest
import pytest_asyncio
from voicevox_core import AudioQuery
from voicevox_core.asyncio import Onnxruntime, OpenJtalk, Synthesizer, VoiceModelFile


@pytest.mark.asyncio
async def test(synthesizer: Synthesizer) -> None:
    TEXT = "こんにちは？"
    STYLE_ID = 0

    wav1 = await synthesizer.tts(TEXT, STYLE_ID)

    query = await synthesizer.create_audio_query(TEXT, STYLE_ID)
    wav2 = await synthesizer.synthesis(query, STYLE_ID)

    phrases = await synthesizer.create_accent_phrases(TEXT, STYLE_ID)
    print(f"{phrases=}")
    print(f"{type(phrases)=}")
    query = AudioQuery.from_accent_phrases(phrases)
    wav3 = await synthesizer.synthesis(query, STYLE_ID)

    phrases = await synthesizer.open_jtalk.analyze(TEXT)
    phrases = await synthesizer.replace_mora_data(phrases, STYLE_ID)
    query = AudioQuery.from_accent_phrases(phrases)
    wav4 = await synthesizer.synthesis(query, STYLE_ID)

    phrases = await synthesizer.open_jtalk.analyze(TEXT)
    phrases = await synthesizer.replace_phoneme_length(phrases, STYLE_ID)
    phrases = await synthesizer.replace_mora_pitch(phrases, STYLE_ID)
    query = AudioQuery.from_accent_phrases(phrases)
    wav5 = await synthesizer.synthesis(query, STYLE_ID)

    wav6 = await synthesizer.tts(TEXT, STYLE_ID, enable_interrogative_upspeak=False)

    query = await synthesizer.create_audio_query(TEXT, STYLE_ID)
    wav7 = await synthesizer.synthesis(
        query, STYLE_ID, enable_interrogative_upspeak=False
    )

    phrases = await synthesizer.create_accent_phrases(TEXT, STYLE_ID)
    query = AudioQuery.from_accent_phrases(phrases)
    wav8 = await synthesizer.synthesis(
        query, STYLE_ID, enable_interrogative_upspeak=False
    )

    phrases = await synthesizer.open_jtalk.analyze(TEXT)
    phrases = await synthesizer.replace_mora_data(phrases, STYLE_ID)
    query = AudioQuery.from_accent_phrases(phrases)
    wav9 = await synthesizer.synthesis(
        query, STYLE_ID, enable_interrogative_upspeak=False
    )

    phrases = await synthesizer.open_jtalk.analyze(TEXT)
    phrases = await synthesizer.replace_phoneme_length(phrases, STYLE_ID)
    phrases = await synthesizer.replace_mora_pitch(phrases, STYLE_ID)
    query = AudioQuery.from_accent_phrases(phrases)
    wav10 = await synthesizer.synthesis(
        query, STYLE_ID, enable_interrogative_upspeak=False
    )

    assert wav1 != wav6
    assert len({wav1, wav2, wav3, wav4, wav5}) == 1
    assert len({wav6, wav7, wav8, wav9, wav10}) == 1


@pytest_asyncio.fixture
async def synthesizer() -> Synthesizer:
    onnxruntime = await Onnxruntime.load_once(filename=conftest.onnxruntime_filename)
    open_jtalk = await OpenJtalk.new(conftest.open_jtalk_dic_dir)
    synthesizer = Synthesizer(
        onnxruntime,
        open_jtalk,
        acceleration_mode="CPU",
        cpu_num_threads=max(
            multiprocessing.cpu_count(), 2
        )  # https://github.com/VOICEVOX/voicevox_core/issues/888
        if platform.system() == "Darwin"
        else 0,  # default
    )
    async with await VoiceModelFile.open(conftest.model_dir) as model:
        await synthesizer.load_voice_model(model)
    return synthesizer
