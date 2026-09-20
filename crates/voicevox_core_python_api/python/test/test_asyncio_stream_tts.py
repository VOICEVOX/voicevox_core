"""
ストリーミング音声合成を行う。

``test_blocking_stream_tts`` と対になる。
"""

import multiprocessing
import platform

import conftest
import pytest
import pytest_asyncio
from voicevox_core import wav_from_s16le
from voicevox_core.asyncio import Onnxruntime, OpenJtalk, Synthesizer, VoiceModelFile


@pytest.mark.asyncio
async def test_render(synthesizer: Synthesizer) -> None:
    TEXT = "こんにちは？"
    # `streaming_talk`に対応したスタイルを使用。voicevox_core/model/sample.vvm/metas.jsonを参照。
    STYLE_ID = 302

    wav1 = await synthesizer.tts(TEXT, STYLE_ID)

    query = await synthesizer.create_audio_query(TEXT, STYLE_ID)
    feat = await synthesizer.create_audio_feature(query, STYLE_ID)
    pcm = await synthesizer.render(feat, 0, feat.frame_length)
    wav2 = wav_from_s16le(pcm, query.output_sampling_rate, query.output_stereo)

    assert wav1 == wav2


@pytest.mark.asyncio
async def test_streaming_synthesis(synthesizer: Synthesizer) -> None:
    TEXT = "こんにちは？"
    STYLE_ID = 302

    wav1 = await synthesizer.tts(TEXT, STYLE_ID)

    wav_stream = await synthesizer.streaming_synthesis(TEXT, STYLE_ID)
    wav2 = b""
    async for chunk in wav_stream:
        wav2 += chunk

    assert wav1 == wav2


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
