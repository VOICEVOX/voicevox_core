import numpy as np
from conftest import ExampleData
from voicevox_core import VoicevoxCore


# crates/voicevox_core_c_api/tests/e2e/testcases/compatible_engine.rs と同じ。
# crates/voicevox_core_c_api/tests/e2e/testcases/compatible_engine_load_model_before_initialize.rs
# （コア初期化前にモデルをロードするとエラーになる）は、コアを初期化せずにモデルをロードすることが出来ないため、
# Python API版でのテストはしない。
def test_engine(example_data: ExampleData):
    core = VoicevoxCore()
    speaker_id = example_data["speaker_id"]
    assert not core.is_model_loaded(speaker_id)
    core.load_model(speaker_id)
    assert core.is_model_loaded(speaker_id)

    duration = core.predict_duration(
        np.array(example_data["duration"]["phoneme_vector"], dtype=np.int64),
        speaker_id,
    )

    intonation = core.predict_intonation(
        example_data["intonation"]["length"],
        np.array(example_data["intonation"]["vowel_phoneme_vector"], dtype=np.int64),
        np.array(
            example_data["intonation"]["consonant_phoneme_vector"], dtype=np.int64
        ),
        np.array(example_data["intonation"]["start_accent_vector"], dtype=np.int64),
        np.array(example_data["intonation"]["end_accent_vector"], dtype=np.int64),
        np.array(
            example_data["intonation"]["start_accent_phrase_vector"], dtype=np.int64
        ),
        np.array(
            example_data["intonation"]["end_accent_phrase_vector"], dtype=np.int64
        ),
        speaker_id,
    )

    wave = core.decode(
        example_data["decode"]["f0_length"],
        example_data["decode"]["phoneme_size"],
        np.array(example_data["decode"]["f0_vector"], dtype=np.float32),
        np.array(example_data["decode"]["phoneme_vector"], dtype=np.float32),
        speaker_id,
    )

    check_float_array_near(
        duration, np.array(example_data["duration"]["result"], dtype=np.float32), 0.01
    )
    check_float_array_near(
        intonation,
        np.array(example_data["intonation"]["result"], dtype=np.float32),
        0.01,
    )

    assert not np.isnan(wave).any()
    assert not np.isinf(wave).any()


def check_float_array_near(a: np.ndarray, b: np.ndarray, max_abs_diff: float):
    assert a.dtype == b.dtype
    assert a.shape == b.shape
    assert np.max(np.abs(a - b)) <= max_abs_diff
