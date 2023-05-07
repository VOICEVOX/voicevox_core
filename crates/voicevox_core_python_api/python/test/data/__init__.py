import numpy as np
import os
from pathlib import Path

data_dir = Path(os.path.dirname(os.path.abspath(__file__)))

# 「こんにちは、音声合成の世界へようこそ」という文章を変換して得た phoneme_vector
example_phoneme_vector = np.load(data_dir / "predict_duration.npy")

# 「テスト」という文章に対応する入力
predict_intonation_data = np.load(data_dir / "predict_intonation.npz")
example_vowel_phoneme_vector = predict_intonation_data["vowel_phoneme_vector"]
example_consonant_phoneme_vector = predict_intonation_data["consonant_phoneme_vector"]
example_start_accent_vector = predict_intonation_data["start_accent_vector"]
example_end_accent_vector = predict_intonation_data["end_accent_vector"]
example_start_accent_phrase_vector = predict_intonation_data[
    "start_accent_phrase_vector"
]
example_end_accent_phrase_vector = predict_intonation_data["end_accent_phrase_vector"]


TEXT_CONSONANT_VOWEL_DATA1 = [
    ([("コ", "k", "o"), ("レ", "r", "e"), ("ワ", "w", "a")], 3),
    (
        [
            ("テ", "t", "e"),
            ("ス", "s", "U"),
            ("ト", "t", "o"),
            ("デ", "d", "e"),
            ("ス", "s", "U"),
        ],
        1,
    ),
]

TEXT_CONSONANT_VOWEL_DATA2 = [
    ([("コ", "k", "o"), ("レ", "r", "e"), ("ワ", "w", "a")], 1),
    (
        [
            ("テ", "t", "e"),
            ("ス", "s", "U"),
            ("ト", "t", "o"),
            ("デ", "d", "e"),
            ("ス", "s", "U"),
        ],
        3,
    ),
]

decode_data = np.load(data_dir / "decode.npz")
example_f0 = decode_data["f0"]
example_phoneme = decode_data["phoneme"]
example_f0_length = 69
example_phoneme_size = 45
