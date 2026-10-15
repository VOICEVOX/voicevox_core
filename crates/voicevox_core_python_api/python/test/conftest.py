import asyncio
import json
import os
from asyncio import AbstractEventLoop
from collections.abc import Generator
from pathlib import Path
from typing import TypedDict

import pytest
import pytest_asyncio
import voicevox_core

root_dir = Path(os.path.dirname(os.path.abspath(__file__)))

onnxruntime_filename = str(
    root_dir.parent.parent.parent
    / "test_util"
    / "data"
    / "lib"
    / voicevox_core.blocking.Onnxruntime.LIB_RECOMMENDED_UNVERSIONED_FILENAME.replace(
        "voicevox_onnxruntime", "onnxruntime"
    )
)
open_jtalk_dic_dir = (
    root_dir.parent.parent.parent / "test_util" / "data" / "open_jtalk_dic_utf_8-1.11"
)
model_dir = (
    root_dir.parent.parent.parent / "test_util" / "data" / "model" / "sample.vvm"
)


class DurationExampleData(TypedDict):
    length: int
    phoneme_vector: list[int]
    result: list[float]


class IntonationExampleData(TypedDict):
    length: int
    vowel_phoneme_vector: list[int]
    consonant_phoneme_vector: list[int]
    start_accent_vector: list[int]
    end_accent_vector: list[int]
    start_accent_phrase_vector: list[int]
    end_accent_phrase_vector: list[int]
    result: list[float]


class DecodeExampleData(TypedDict):
    f0_length: int
    phoneme_size: int
    f0_vector: list[float]
    phoneme_vector: list[float]


class ExampleData(TypedDict):
    speaker_id: int
    duration: DurationExampleData
    intonation: IntonationExampleData
    decode: DecodeExampleData


@pytest.fixture(scope="session")
def example_data() -> ExampleData:
    with (
        root_dir.parent.parent.parent / "test_util" / "data" / "example_data.json"
    ).open() as f:
        return json.load(f)


@pytest.fixture(scope="session")
def blocking_synthesizesr_with_model() -> voicevox_core.blocking.Synthesizer:
    from voicevox_core.blocking import (
        Onnxruntime,
        OpenJtalk,
        Synthesizer,
        VoiceModelFile,
    )

    ort = Onnxruntime.load_once(filename=onnxruntime_filename)
    ojt = OpenJtalk(open_jtalk_dic_dir)
    synth = Synthesizer(ort, ojt, acceleration_mode="CPU")
    with VoiceModelFile.open(model_dir) as vvm:
        synth.load_voice_model(vvm)
    return synth


# pyright: reportUnknownMemberType=false, reportUntypedFunctionDecorator=false
@pytest_asyncio.fixture(scope="session")
async def asyncio_synthesizesr_with_model() -> voicevox_core.asyncio.Synthesizer:
    from voicevox_core.asyncio import (
        Onnxruntime,
        OpenJtalk,
        Synthesizer,
        VoiceModelFile,
    )

    ort = await Onnxruntime.load_once(filename=onnxruntime_filename)
    ojt = await OpenJtalk.new(open_jtalk_dic_dir)
    synth = Synthesizer(ort, ojt, acceleration_mode="CPU")
    async with await VoiceModelFile.open(model_dir) as vvm:
        await synth.load_voice_model(vvm)
    return synth


@pytest.fixture(scope="session")
def event_loop() -> Generator[AbstractEventLoop, object]:
    loop = asyncio.new_event_loop()
    yield loop
    loop.close()
