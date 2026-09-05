"""
ストリーミング音声合成を行う。
"""

import conftest
import pytest
from voicevox_core import wav_from_s16le
from voicevox_core.blocking import Onnxruntime, OpenJtalk, Synthesizer, VoiceModelFile


def test(synthesizer: Synthesizer) -> None:
    TEXT = "こんにちは？"
    # `streaming_talk`に対応したスタイルを使用。voicevox_core/model/sample.vvm/metas.jsonを参照。
    STYLE_ID = 302

    wav1 = synthesizer.tts(TEXT, STYLE_ID)

    query = synthesizer.create_audio_query(TEXT, STYLE_ID)
    feat = synthesizer.create_audio_feature(query, STYLE_ID)
    pcm = synthesizer.render(feat, 0, feat.frame_length)
    wav2 = wav_from_s16le(pcm, query.output_sampling_rate, query.output_stereo)

    assert wav1 == wav2


@pytest.fixture
def synthesizer() -> Synthesizer:
    onnxruntime = Onnxruntime.load_once(filename=conftest.onnxruntime_filename)
    open_jtalk = OpenJtalk(conftest.open_jtalk_dic_dir)
    synthesizer = Synthesizer(onnxruntime, open_jtalk, acceleration_mode="CPU")
    with VoiceModelFile.open(conftest.model_dir) as model:
        synthesizer.load_voice_model(model)
    return synthesizer
