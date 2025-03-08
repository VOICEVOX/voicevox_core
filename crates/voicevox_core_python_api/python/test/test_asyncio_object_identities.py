"""
同一であるべきオブジェクトと同一であるべきではないオブジェクトについて、同一性
(*identity*)を確認する。

``test_blocking_object_identities`` と対になる。
"""

import conftest
import pytest
from voicevox_core.asyncio import (
    Onnxruntime,
    OpenJtalk,
    Synthesizer,
    UserDict,
    VoiceModelFile,
)


@pytest.mark.asyncio
async def test() -> None:
    onnxruntime = await Onnxruntime.load_once(filename=conftest.onnxruntime_filename)

    assert Onnxruntime.get() is onnxruntime
    assert await Onnxruntime.load_once() is onnxruntime
    assert onnxruntime.supported_devices() is not onnxruntime.supported_devices()

    open_jtalk = await OpenJtalk.new(conftest.open_jtalk_dic_dir)

    synthesizer = Synthesizer(onnxruntime, open_jtalk)
    assert synthesizer.onnxruntime is onnxruntime
    assert synthesizer.open_jtalk is open_jtalk
    assert synthesizer.metas() is not synthesizer.metas()

    async with await VoiceModelFile.open(conftest.model_dir) as model:
        assert model.id is model.id
        assert model.metas is model.metas

    userdict = UserDict()
    assert userdict.to_dict() is not userdict.to_dict()
