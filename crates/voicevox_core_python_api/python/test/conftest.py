import json
import os
from dataclasses import dataclass
from pathlib import Path
from typing import List, TypedDict

import numpy as np
import pytest

# onnxruntimeを最初に読み込んでおく
if ort_path := os.getenv("ORT_PATH"):
    import ctypes

    ctypes.cdll.LoadLibrary(ort_path)

root_dir = Path(os.path.dirname(os.path.abspath(__file__)))


class DurationExampleData(TypedDict):
    length: int
    phoneme_vector: List[int]
    result: List[float]


class IntonationExampleData(TypedDict):
    length: int
    vowel_phoneme_vector: List[int]
    consonant_phoneme_vector: List[int]
    start_accent_vector: List[int]
    end_accent_vector: List[int]
    start_accent_phrase_vector: List[int]
    end_accent_phrase_vector: List[int]
    result: List[float]


class DecodeExampleData(TypedDict):
    f0_length: int
    phoneme_size: int
    f0_vector: List[float]
    phoneme_vector: List[float]


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
