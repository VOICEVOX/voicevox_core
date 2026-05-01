"""
歌唱音声合成を行う。

``test_asyncio_song`` と対になる。
"""

import conftest
import pytest
from voicevox_core import Note, NoteId, Score, StyleId
from voicevox_core.blocking import Onnxruntime, OpenJtalk, Synthesizer, VoiceModelFile


def test(synthesizer: Synthesizer) -> None:
    SINGING_TEACHER = StyleId(6000)
    SINGER = StyleId(3000)

    SCORE = Score(
        [
            Note(15, "", id=NoteId("①")),
            Note(45, "ド", key=60, id=NoteId("②")),
            Note(45, "レ", key=62, id=NoteId("③")),
            Note(45, "ミ", key=64, id=NoteId("④")),
            Note(15, "", id=NoteId("⑤")),
        ],
    )

    NUM_TOTAL_FRAMES = sum(note.frame_length for note in SCORE.notes)

    frame_audio_query = synthesizer.create_sing_frame_audio_query(
        SCORE, SINGING_TEACHER
    )

    phonemes = [phoneme.phoneme for phoneme in frame_audio_query.phonemes]
    assert phonemes == ["pau", "d", "o", "r", "e", "m", "i", "pau"]

    note_ids = [phoneme.note_id for phoneme in frame_audio_query.phonemes]
    assert note_ids == ["①", "②", "②", "③", "③", "④", "④", "⑤"]

    assert all(
        n == NUM_TOTAL_FRAMES
        for n in [
            sum(phoneme.frame_length for phoneme in frame_audio_query.phonemes),
            len(frame_audio_query.f0),
            len(frame_audio_query.volume),
        ]
    )

    f0s = synthesizer.create_sing_frame_f0(SCORE, frame_audio_query, SINGING_TEACHER)
    assert len(f0s) == NUM_TOTAL_FRAMES

    volumes = synthesizer.create_sing_frame_volume(
        SCORE, frame_audio_query, SINGING_TEACHER
    )
    assert len(volumes) == NUM_TOTAL_FRAMES

    wav = synthesizer.frame_synthesis(frame_audio_query, SINGER)

    assert wav.startswith(b"RIFF")
    assert (
        NUM_TOTAL_FRAMES * 256 * 2 * (1 + frame_audio_query.output_stereo)
        == int.from_bytes(wav[4:8], "little") - 36
    )
    assert wav[8:16] == b"WAVEfmt "


@pytest.fixture
def synthesizer() -> Synthesizer:
    onnxruntime = Onnxruntime.load_once(filename=conftest.onnxruntime_filename)
    open_jtalk = OpenJtalk(conftest.open_jtalk_dic_dir)
    synthesizer = Synthesizer(onnxruntime, open_jtalk, acceleration_mode="CPU")
    with VoiceModelFile.open(conftest.model_dir) as model:
        synthesizer.load_voice_model(model)
    return synthesizer
