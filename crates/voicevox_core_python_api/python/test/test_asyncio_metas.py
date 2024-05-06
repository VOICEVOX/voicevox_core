"""
メタ情報の出力が可能かどうかをテストする。

``test_blocking_metas`` と対になる。
"""

import conftest
import pytest
import pytest_asyncio
from voicevox_core.asyncio import OpenJtalk, Synthesizer, VoiceModel


def test_voice_model_metas_works(voice_model: VoiceModel) -> None:
    _ = voice_model.metas


@pytest.mark.asyncio
async def test_synthesizer_metas_works(voice_model: VoiceModel) -> None:
    synthesizer = Synthesizer(await OpenJtalk.new(conftest.open_jtalk_dic_dir))
    await synthesizer.load_voice_model(voice_model)
    _ = synthesizer.metas


@pytest_asyncio.fixture
async def voice_model() -> VoiceModel:
    return await VoiceModel.from_path(conftest.model_dir)
