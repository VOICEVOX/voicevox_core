import numpy as np
from voicevox_core import VoicevoxCore
from data import (
    example_f0_length,
    example_phoneme_size,
    example_f0,
    example_phoneme,
    example_duration,
    example_intonation,
)

SPEAKER_ID = 0


# crates/voicevox_core_c_api/tests/e2e/testcases/compatible_engine.rs と同じ。
# crates/voicevox_core_c_api/tests/e2e/testcases/compatible_engine_load_model_before_initialize.rs
# （コア初期化前にモデルをロードするとエラーになる）は、コアを初期化せずにモデルをロードすることが出来ないため、
# Python API版でのテストはしない。
def test_engine():
    core = VoicevoxCore()
    assert not core.is_model_loaded(SPEAKER_ID)
    core.load_model(SPEAKER_ID)
    assert core.is_model_loaded(SPEAKER_ID)

    duration = core.predict_duration(
        # 「t e s u t o」
        np.array([0, 37, 14, 35, 6, 37, 30, 0], dtype=np.int64),
        SPEAKER_ID,
    )

    intonation = core.predict_intonation(
        5,
        np.array([0, 14, 6, 30, 0], dtype=np.int64),
        np.array([-1, 37, 35, 37, -1], dtype=np.int64),
        np.array([0, 1, 0, 0, 0], dtype=np.int64),
        np.array([0, 1, 0, 0, 0], dtype=np.int64),
        np.array([0, 1, 0, 0, 0], dtype=np.int64),
        np.array([0, 0, 0, 1, 0], dtype=np.int64),
        SPEAKER_ID,
    )

    wave = core.decode(
        example_f0_length,
        example_phoneme_size,
        example_f0,
        example_phoneme,
        SPEAKER_ID,
    )

    check_float_array_near(duration, example_duration, 0.01)
    check_float_array_near(intonation, example_intonation, 0.01)

    assert not np.isnan(wave).any()
    assert not np.isinf(wave).any()


def check_float_array_near(a: np.ndarray, b: np.ndarray, max_abs_diff: float):
    assert a.dtype == b.dtype
    assert a.shape == b.shape
    assert np.max(np.abs(a - b)) <= max_abs_diff
