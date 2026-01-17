import json
import os
from pathlib import Path
from typing import TypedDict

import pytest
import voicevox_core

root_dir = Path(os.path.dirname(os.path.abspath(__file__)))

onnxruntime_filename = str(
    root_dir.parent.parent.parent.parent
    / "target"
    / "voicevox_core"
    / "downloads"
    / "onnxruntime"
    / voicevox_core.blocking.Onnxruntime.LIB_VERSIONED_FILENAME.replace(
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
