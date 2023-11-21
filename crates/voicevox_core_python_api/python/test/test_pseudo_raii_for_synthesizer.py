"""
``Synthesizer`` について、(広義の)RAIIができることをテストする。
"""

import conftest
import pytest
import pytest_asyncio
from voicevox_core import OpenJtalk, Synthesizer


def test_enter_returns_workable_self(synthesizer: Synthesizer) -> None:
    with synthesizer as ctx:
        assert ctx is synthesizer
        _ = synthesizer.metas


def test_closing_multiple_times_is_allowed(synthesizer: Synthesizer) -> None:
    with synthesizer:
        with synthesizer:
            pass
    synthesizer.close()
    synthesizer.close()


def test_access_after_close_denied(synthesizer: Synthesizer) -> None:
    synthesizer.close()
    with pytest.raises(ValueError, match="^The `Synthesizer` is closed$"):
        _ = synthesizer.metas


def test_access_after_exit_denied(synthesizer: Synthesizer) -> None:
    with synthesizer:
        pass
    with pytest.raises(ValueError, match="^The `Synthesizer` is closed$"):
        _ = synthesizer.metas


@pytest_asyncio.fixture
async def synthesizer(open_jtalk: OpenJtalk) -> Synthesizer:
    return Synthesizer(open_jtalk)


@pytest_asyncio.fixture(scope="module")
async def open_jtalk() -> OpenJtalk:
    return await OpenJtalk.new(conftest.open_jtalk_dic_dir)
