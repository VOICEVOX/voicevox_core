from itertools import product

from voicevox_core import FRAME_RATE, AudioQuery
from voicevox_core.blocking import Synthesizer


def test_slicing_half_closed(
    blocking_synthesizer_with_model: Synthesizer,
) -> None:
    LEN = 10
    STYLE_ID = 302

    query = AudioQuery.from_accent_phrases([])
    query.pre_phoneme_length = LEN / FRAME_RATE
    query.post_phoneme_length = 0.0
    feat = blocking_synthesizer_with_model.create_audio_feature(query, STYLE_ID)
    assert len(feat) == LEN

    for i, j, k, l in product(range(-LEN - 1, LEN + 2), repeat=4):
        expected = range(LEN)[i:j][k:l]
        actual = feat[i:j][k:l]
        assert (actual.start_offset, len(actual)) == (expected.start, len(expected))
