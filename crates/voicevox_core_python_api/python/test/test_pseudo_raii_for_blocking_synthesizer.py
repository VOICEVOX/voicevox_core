"""
``Synthesizer`` について、(広義の)RAIIができることをテストする。

``test_pseudo_raii_for_asyncio_synthesizer`` と対になる。
"""

import conftest
import pytest
from voicevox_core.blocking import Onnxruntime, OpenJtalk, Synthesizer


def test_enter_returns_workable_self(synthesizer: Synthesizer) -> None:
    with synthesizer as ctx:
        assert ctx is synthesizer
        _ = synthesizer.metas()


def test_closing_multiple_times_is_allowed(synthesizer: Synthesizer) -> None:
    with synthesizer:
        with synthesizer:
            pass
    synthesizer.close()
    synthesizer.close()


def test_access_after_close_denied(synthesizer: Synthesizer) -> None:
    synthesizer.close()
    with pytest.raises(ValueError, match="^The `Synthesizer` is closed$"):
        _ = synthesizer.metas()


def test_access_after_exit_denied(synthesizer: Synthesizer) -> None:
    with synthesizer:
        pass
    with pytest.raises(ValueError, match="^The `Synthesizer` is closed$"):
        _ = synthesizer.metas()


@pytest.fixture
def synthesizer(onnxruntime: Onnxruntime, open_jtalk: OpenJtalk) -> Synthesizer:
    return Synthesizer(onnxruntime, open_jtalk)


@pytest.fixture(scope="session")
def onnxruntime() -> Onnxruntime:
    return Onnxruntime.load_once(filename=conftest.onnxruntime_filename)


@pytest.fixture(scope="session")
def open_jtalk() -> OpenJtalk:
    return OpenJtalk(conftest.open_jtalk_dic_dir)
