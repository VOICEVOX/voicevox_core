"""
同一であるべきオブジェクトと同一であるべきではないオブジェクトについて、同一性
(*identity*)を確認する。

``test_asyncio_object_identities`` と対になる。
"""

import conftest
from voicevox_core.blocking import (
    Onnxruntime,
    OpenJtalk,
    Synthesizer,
    UserDict,
    VoiceModelFile,
)


def test() -> None:
    onnxruntime = Onnxruntime.load_once(filename=conftest.onnxruntime_filename)

    assert Onnxruntime.get() is onnxruntime
    assert Onnxruntime.load_once() is onnxruntime
    assert onnxruntime.supported_devices() is not onnxruntime.supported_devices()

    open_jtalk = OpenJtalk(conftest.open_jtalk_dic_dir)

    synthesizer = Synthesizer(onnxruntime, open_jtalk)
    assert synthesizer.onnxruntime is onnxruntime
    assert synthesizer.open_jtalk is open_jtalk
    assert synthesizer.metas() is not synthesizer.metas()

    with VoiceModelFile.open(conftest.model_dir) as model:
        assert model.id is model.id
        assert model.metas is model.metas

    userdict = UserDict()
    assert userdict.to_dict() is not userdict.to_dict()
