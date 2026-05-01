"""
歌唱音声合成を行う。

``test_blocking_song`` と対になる。
"""

import multiprocessing
import platform

import conftest
import pytest
import pytest_asyncio
from voicevox_core import Note, NoteId, Score, StyleId
from voicevox_core.asyncio import Onnxruntime, OpenJtalk, Synthesizer, VoiceModelFile


@pytest.mark.asyncio
async def test(synthesizer: Synthesizer) -> None:
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

    frame_audio_query = await synthesizer.create_sing_frame_audio_query(
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

    f0s = await synthesizer.create_sing_frame_f0(
        SCORE, frame_audio_query, SINGING_TEACHER
    )
    assert len(f0s) == NUM_TOTAL_FRAMES

    volumes = await synthesizer.create_sing_frame_volume(
        SCORE, frame_audio_query, SINGING_TEACHER
    )
    assert len(volumes) == NUM_TOTAL_FRAMES

    wav = await synthesizer.frame_synthesis(frame_audio_query, SINGER)

    assert wav.startswith(b"RIFF")
    assert (
        NUM_TOTAL_FRAMES * 256 * 2 * (1 + frame_audio_query.output_stereo)
        == int.from_bytes(wav[4:8], "little") - 36
    )
    assert wav[8:16] == b"WAVEfmt "


@pytest_asyncio.fixture
async def synthesizer() -> Synthesizer:
    onnxruntime = await Onnxruntime.load_once(filename=conftest.onnxruntime_filename)
    open_jtalk = await OpenJtalk.new(conftest.open_jtalk_dic_dir)
    synthesizer = Synthesizer(
        onnxruntime,
        open_jtalk,
        acceleration_mode="CPU",
        cpu_num_threads=max(
            multiprocessing.cpu_count(), 2
        )  # https://github.com/VOICEVOX/voicevox_core/issues/888
        if platform.system() == "Darwin"
        else 0,  # default
    )
    async with await VoiceModelFile.open(conftest.model_dir) as model:
        await synthesizer.load_voice_model(model)
    return synthesizer
