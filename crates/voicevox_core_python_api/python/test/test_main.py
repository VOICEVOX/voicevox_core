import pytest
import numpy as np
from voicevox_core import VoicevoxCore
from data import (
    example_phoneme_vector,
    example_vowel_phoneme_vector,
    example_consonant_phoneme_vector,
    example_start_accent_vector,
    example_end_accent_vector,
    example_start_accent_phrase_vector,
    example_end_accent_phrase_vector,
    TEXT_CONSONANT_VOWEL_DATA1,
    TEXT_CONSONANT_VOWEL_DATA2,
    example_f0,
    example_phoneme,
    example_f0_length,
    example_phoneme_size,
)
from typing import List, Tuple
import voicevox_core


def test_initialize(open_jtalk_dict_dir: str):
    VoicevoxCore(open_jtalk_dict_dir=open_jtalk_dict_dir)


def test_decode(core: VoicevoxCore):
    result = core.decode(
        example_f0_length, example_phoneme_size, example_f0, example_phoneme, 0
    )
    assert len(result) == len(example_f0) * 256


def test_tts(core: VoicevoxCore):
    result = core.tts("テストです", 0)
    assert len(result) > 0


def test_synthesis(core: VoicevoxCore):
    query = core.audio_query("テストです", 0, False)
    result = core.synthesis(
        query,
        0,
    )
    assert len(result) > 0


def test_predict_duration(core: VoicevoxCore):
    result = core.predict_duration(example_phoneme_vector, 0)
    assert len(result) == len(example_phoneme_vector)


def test_predict_intonation(core: VoicevoxCore):
    result = core.predict_intonation(
        len(example_vowel_phoneme_vector),
        example_vowel_phoneme_vector,
        example_consonant_phoneme_vector,
        example_start_accent_vector,
        example_end_accent_vector,
        example_start_accent_phrase_vector,
        example_end_accent_phrase_vector,
        0,
    )
    assert len(result) == len(example_vowel_phoneme_vector)


@pytest.mark.parametrize(
    "input_text,input_kana_option,expected_text_consonant_vowel_data,expected_kana_text",
    [
        ("これはテストです", False, TEXT_CONSONANT_VOWEL_DATA1, "コレワ'/テ'_ストデ_ス"),
        ("コ'レワ/テ_スト'デ_ス", True, TEXT_CONSONANT_VOWEL_DATA2, "コ'レワ/テ_スト'デ_ス"),
    ],
)
def test_audio_query(
    core: VoicevoxCore,
    input_text: str,
    input_kana_option: bool,
    expected_text_consonant_vowel_data: List[Tuple[List[Tuple[str, str, str]], int]],
    expected_kana_text: str,
):
    result = core.audio_query(input_text, 0, input_kana_option)
    check_accent_phrases(result.accent_phrases, expected_text_consonant_vowel_data)

    assert result.kana == expected_kana_text


@pytest.mark.parametrize(
    "input_text,input_kana_option,expected_text_consonant_vowel_data",
    [
        ("これはテストです", False, TEXT_CONSONANT_VOWEL_DATA1),
        ("コ'レワ/テ_スト'デ_ス", True, TEXT_CONSONANT_VOWEL_DATA2),
    ],
)
def test_accent_phrases(
    core: VoicevoxCore,
    input_text: str,
    input_kana_option: bool,
    expected_text_consonant_vowel_data: List[Tuple[List[Tuple[str, str, str]], int]],
):
    result = core.accent_phrases(input_text, 0, input_kana_option)
    check_accent_phrases(result, expected_text_consonant_vowel_data)


def check_accent_phrases(
    accent_phrases: List[voicevox_core.AccentPhrase],
    expected_text_consonant_vowel_data: List[Tuple[List[Tuple[str, str, str]], int]],
):
    assert len(accent_phrases) == len(expected_text_consonant_vowel_data)
    for accent_phrase, (text_consonant_vowels, accent_pos) in zip(
        accent_phrases, expected_text_consonant_vowel_data
    ):
        assert len(accent_phrase.moras) == len(text_consonant_vowels)
        assert accent_phrase.accent == accent_pos
        for mora, (text, consonant, vowel) in zip(
            accent_phrase.moras, text_consonant_vowels
        ):
            assert mora.text == text
            assert mora.consonant == consonant
            assert mora.vowel == vowel

            # 母音・子音の長さが0以上になるテストケースを想定している
            if mora.consonant_length is not None:
                assert mora.consonant_length > 0
            assert mora.vowel_length > 0


def test_mora_length(core: VoicevoxCore):
    accent_phrases = core.accent_phrases("これはテストです", 0, False)
    with_mora_length_replaced = core.mora_length(accent_phrases, 1)

    # 一つでも母音の長さが変わっていれば、動作しているとみなす
    check_any_mora_param_changed(
        accent_phrases, with_mora_length_replaced, "vowel_length"
    )


def test_mora_pitch(core: VoicevoxCore):
    accent_phrases = core.accent_phrases("これはテストです", 0, False)
    with_mora_length_replaced = core.mora_pitch(accent_phrases, 1)

    # 一つでも音高が変わっていれば、動作しているとみなす
    check_any_mora_param_changed(accent_phrases, with_mora_length_replaced, "pitch")


def test_mora_data(core: VoicevoxCore):
    accent_phrases = core.accent_phrases("これはテストです", 0, False)
    with_mora_length_replaced = core.mora_data(accent_phrases, 1)


    check_any_mora_param_changed(accent_phrases, with_mora_length_replaced, "pitch")
    check_any_mora_param_changed(
        accent_phrases, with_mora_length_replaced, "vowel_length"
    )


def check_any_mora_param_changed(
    before: List[voicevox_core.AccentPhrase],
    after: List[voicevox_core.AccentPhrase],
    attr: str,
):
    assert len(before) == len(after)
    for before_accent_phrase, after_accent_phrase in zip(before, after):
        for before_mora, after_mora in zip(
            before_accent_phrase.moras, after_accent_phrase.moras
        ):
            if getattr(before_mora, attr) != getattr(after_mora, attr):
                return

    assert False, f"no mora.{attr} changed"
