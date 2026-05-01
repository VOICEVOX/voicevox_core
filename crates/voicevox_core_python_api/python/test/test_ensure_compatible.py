import pytest
import voicevox_core
from voicevox_core import FrameAudioQuery, FramePhoneme, Note, Score


def test_compatible() -> None:
    voicevox_core.ensure_compatible(
        Score(
            [
                note(""),
                note("ド", key=0),
                note("レ", key=0),
                note("ミ", key=0),
                note(""),
            ],
        ),
        frame_audio_query(
            [
                frame_phoneme("pau"),
                frame_phoneme("d"),
                frame_phoneme("o"),
                frame_phoneme("r"),
                frame_phoneme("e"),
                frame_phoneme("m"),
                frame_phoneme("i"),
                frame_phoneme("pau"),
            ],
        ),
    )


def test_incompatible() -> None:
    with pytest.raises(voicevox_core.IncompatibleQueriesError):
        voicevox_core.ensure_compatible(
            Score([note(""), note("ア", key=0)]),
            frame_audio_query([frame_phoneme("pau"), frame_phoneme("i")]),
        )


def note(lyric: str, *, key: int | None = None) -> Note:
    return Note(0, lyric, key=key)


def frame_phoneme(phoneme: str) -> FramePhoneme:
    return FramePhoneme(phoneme, 0)


def frame_audio_query(phonemes: list[FramePhoneme]) -> FrameAudioQuery:
    return FrameAudioQuery([], [], phonemes, 1.0, 24000, False)
