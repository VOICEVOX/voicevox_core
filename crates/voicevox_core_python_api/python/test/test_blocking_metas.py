"""
メタ情報の出力が可能かどうかをテストする。

``test_asyncio_metas`` と対になる。
"""

import conftest
import pytest
from voicevox_core.blocking import OpenJtalk, Synthesizer, VoiceModel


def test_voice_model_metas_works(voice_model: VoiceModel) -> None:
    _ = voice_model.metas


def test_synthesizer_metas_works(voice_model: VoiceModel) -> None:
    synthesizer = Synthesizer(OpenJtalk(conftest.open_jtalk_dic_dir))
    synthesizer.load_voice_model(voice_model)
    _ = synthesizer.metas


@pytest.fixture
def voice_model() -> VoiceModel:
    return VoiceModel.from_path(conftest.model_dir)
