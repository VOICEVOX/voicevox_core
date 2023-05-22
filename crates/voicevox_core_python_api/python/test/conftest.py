from dataclasses import dataclass
import numpy as np
import os
from pathlib import Path
import pytest
import requests
import tarfile
import toml

# onnxruntimeを最初に読み込んでおく
if ort_path := os.getenv("ORT_PATH"):
    import ctypes

    ctypes.cdll.LoadLibrary(ort_path)

root_dir = Path(os.path.dirname(os.path.abspath(__file__)))


@pytest.fixture(scope="session")
def open_jtalk_dict_dir():
    if os.path.exists(root_dir / "open_jtalk_dic_utf_8-1.11"):
        return root_dir / "open_jtalk_dic_utf_8-1.11"
    response = requests.get(
        "https://github.com/r9y9/open_jtalk/releases/download/v1.11.1/open_jtalk_dic_utf_8-1.11.tar.gz"
    )
    response.raise_for_status()
    with open(root_dir / "open_jtalk_dic_utf_8-1.11.tar.gz", "wb") as f:
        f.write(response.content)
    with tarfile.open(root_dir / "open_jtalk_dic_utf_8-1.11.tar.gz") as tar:
        tar.extractall(root_dir)
    os.remove(root_dir / "open_jtalk_dic_utf_8-1.11.tar.gz")
    return root_dir / "open_jtalk_dic_utf_8-1.11"


@dataclass
class ExampleData:
    f0: np.ndarray
    phoneme: np.ndarray
    duration: np.ndarray
    intonation: np.ndarray
    f0_length: int
    phoneme_size: int


@pytest.fixture(scope="session")
def example_data():
    example_f0_length = 69
    example_phoneme_size = 45

    example_f0 = np.zeros((example_f0_length,), dtype=np.float32)
    example_f0[9:24] = 5.905128
    example_f0[37:60] = 5.565851
    example_phoneme = np.zeros(
        (example_phoneme_size * example_f0_length,), dtype=np.float32
    )
    for index, range_start, range_end in [
        (0, 0, 9),
        (37, 9, 13),
        (14, 13, 24),
        (35, 24, 30),
        (6, 30, 37),
        (37, 37, 45),
        (30, 45, 60),
        (0, 60, 69),
    ]:
        for i in range(range_start, range_end):
            example_phoneme[i * example_phoneme_size + index] = 1.0

    snapshots = toml.load(
        root_dir.parent.parent.parent
        / "voicevox_core_c_api"
        / "tests"
        / "e2e"
        / "snapshots.toml"
    )

    example_duration = np.array(
        snapshots["compatible_engine"]["yukarin_s_forward"], dtype=np.float32
    )
    example_intonation = np.array(
        snapshots["compatible_engine"]["yukarin_sa_forward"], dtype=np.float32
    )

    return ExampleData(
        f0=example_f0,
        phoneme=example_phoneme,
        duration=example_duration,
        intonation=example_intonation,
        f0_length=example_f0_length,
        phoneme_size=example_phoneme_size,
    )
