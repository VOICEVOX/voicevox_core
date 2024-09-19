"""
メタ情報の出力が可能かどうかをテストする。

``test_blocking_metas`` と対になる。
"""

import conftest
import pytest
import pytest_asyncio
from voicevox_core.asyncio import Onnxruntime, OpenJtalk, Synthesizer, VoiceModelFile


def test_voice_model_metas_works(voice_model: VoiceModelFile) -> None:
    _ = voice_model.metas


@pytest.mark.asyncio
async def test_synthesizer_metas_works(voice_model: VoiceModelFile) -> None:
    synthesizer = Synthesizer(
        await Onnxruntime.load_once(filename=conftest.onnxruntime_filename),
        await OpenJtalk.new(conftest.open_jtalk_dic_dir),
    )
    await synthesizer.load_voice_model(voice_model)
    _ = synthesizer.metas


@pytest_asyncio.fixture
async def voice_model() -> VoiceModelFile:
    return await VoiceModelFile.open(conftest.model_dir)
