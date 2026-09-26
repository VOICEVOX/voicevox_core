import os
from pathlib import Path

import pytest

from .blocking import Onnxruntime, OpenJtalk, Synthesizer, VoiceModelFile


@pytest.fixture(scope="session", autouse=True)
def setup_for_doctests(doctest_namespace: dict[str, object]):
    # FIXME: <../test/conftest.py>のコピペになっているので共通化、
    # というよりfixture化する。
    root_dir = Path(os.path.dirname(os.path.abspath(__file__)))
    onnxruntime_filename = str(
        root_dir.parent.parent.parent
        / "test_util"
        / "data"
        / "lib"
        / Onnxruntime.LIB_RECOMMENDED_UNVERSIONED_FILENAME.replace(
            "voicevox_onnxruntime", "onnxruntime"
        )
    )
    open_jtalk_dic_dir = (
        root_dir.parent.parent.parent
        / "test_util"
        / "data"
        / "open_jtalk_dic_utf_8-1.11"
    )
    model_dir = (
        root_dir.parent.parent.parent / "test_util" / "data" / "model" / "sample.vvm"
    )

    onnxruntime = Onnxruntime.load_once(filename=onnxruntime_filename)
    open_jtalk = OpenJtalk(open_jtalk_dic_dir)
    synth = Synthesizer(onnxruntime, open_jtalk, acceleration_mode="CPU")
    with VoiceModelFile.open(model_dir) as model:
        synth.load_voice_model(model)

    doctest_namespace.update(
        {
            "WHATEVER_STYLE1": 0,
            "WHATEVER_STYLE2": 302,
            "synth": synth,
        }
    )
