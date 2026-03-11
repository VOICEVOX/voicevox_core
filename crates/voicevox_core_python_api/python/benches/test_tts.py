"""GHAで動かすベンチマーク。結果はCodSpeedにアップロードされる。"""

import asyncio
from pathlib import Path

import pytest
import pytest_asyncio
import voicevox_core
from pytest_codspeed import BenchmarkFixture
from voicevox_core import StyleId

ONNXRUNTIME_FILENAME = str(
    Path(__file__).parent.parent.parent.parent.parent
    / "target"
    / "voicevox_core"
    / "downloads"
    / "onnxruntime"
    / voicevox_core.blocking.Onnxruntime.LIB_VERSIONED_FILENAME.replace(
        "voicevox_onnxruntime", "onnxruntime"
    )
)
OPEN_JTALK_DIC_DIR = (
    Path(__file__).parent.parent.parent.parent
    / "test_util"
    / "data"
    / "open_jtalk_dic_utf_8-1.11"
)
VVM_PATH = (
    Path(__file__).parent.parent.parent.parent
    / "test_util"
    / "data"
    / "model"
    / "sample.vvm"
)

TEXT = "この音声は、ボイスボックスを使用して、出力されています。"
STYLE_ID = StyleId(0)


@pytest.mark.benchmark
def test_blocking_tts(
    benchmark: BenchmarkFixture,
    blocking_synthesizer: voicevox_core.blocking.Synthesizer,
):
    synth = blocking_synthesizer

    def tts() -> object:
        return synth.tts(TEXT, STYLE_ID)

    benchmark.pedantic(tts, rounds=10, warmup_rounds=2)


@pytest.fixture
def blocking_synthesizer() -> voicevox_core.blocking.Synthesizer:
    from voicevox_core.blocking import (
        Onnxruntime,
        OpenJtalk,
        Synthesizer,
        VoiceModelFile,
    )

    ort = Onnxruntime.load_once(filename=ONNXRUNTIME_FILENAME)
    ojt = OpenJtalk(OPEN_JTALK_DIC_DIR)
    synth = Synthesizer(ort, ojt)
    with VoiceModelFile.open(VVM_PATH) as vvm:
        synth.load_voice_model(vvm)
    return synth


@pytest.mark.benchmark
def test_asyncio_tts(
    benchmark: BenchmarkFixture,
    asyncio_synthesizer: voicevox_core.asyncio.Synthesizer,
):
    synth = asyncio_synthesizer

    def tts() -> object:
        return asyncio.run(synth.tts(TEXT, STYLE_ID))

    benchmark.pedantic(tts, rounds=10, warmup_rounds=2)


@pytest_asyncio.fixture
async def asyncio_synthesizer() -> voicevox_core.asyncio.Synthesizer:
    from voicevox_core.asyncio import (
        Onnxruntime,
        OpenJtalk,
        Synthesizer,
        VoiceModelFile,
    )

    ort = await Onnxruntime.load_once(filename=ONNXRUNTIME_FILENAME)
    ojt = await OpenJtalk.new(OPEN_JTALK_DIC_DIR)
    synth = Synthesizer(ort, ojt, acceleration_mode="CPU")
    async with await VoiceModelFile.open(VVM_PATH) as vvm:
        await synth.load_voice_model(vvm)
    return synth
