"""
メタ情報の出力が可能かどうかをテストする。

``test_asyncio_metas`` と対になる。
"""

import conftest
import pytest
from voicevox_core.blocking import Onnxruntime, OpenJtalk, Synthesizer, VoiceModelFile


def test_voice_model_metas_works(voice_model: VoiceModelFile) -> None:
    _ = voice_model.metas


def test_synthesizer_metas_works(voice_model: VoiceModelFile) -> None:
    synthesizer = Synthesizer(
        Onnxruntime.load_once(filename=conftest.onnxruntime_filename),
        OpenJtalk(conftest.open_jtalk_dic_dir),
    )
    synthesizer.load_voice_model(voice_model)
    _ = synthesizer.metas


@pytest.fixture
def voice_model() -> VoiceModelFile:
    return VoiceModelFile.open(conftest.model_dir)
