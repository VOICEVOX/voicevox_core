"""
``Synthesizer`` について、(広義の)RAIIができることをテストする。

``test_pseudo_raii_for_blocking_synthesizer`` と対になる。
"""

import conftest
import pytest
import pytest_asyncio
from voicevox_core.asyncio import Onnxruntime, OpenJtalk, Synthesizer


@pytest.mark.asyncio
async def test_enter_returns_workable_self(synthesizer: Synthesizer) -> None:
    async with synthesizer as ctx:
        assert ctx is synthesizer
        _ = synthesizer.metas()


@pytest.mark.asyncio
async def test_closing_multiple_times_is_allowed(synthesizer: Synthesizer) -> None:
    async with synthesizer:
        async with synthesizer:
            pass
    await synthesizer.close()
    await synthesizer.close()


@pytest.mark.asyncio
async def test_access_after_close_denied(synthesizer: Synthesizer) -> None:
    await synthesizer.close()
    with pytest.raises(ValueError, match="^The `Synthesizer` is closed$"):
        _ = synthesizer.metas()


@pytest.mark.asyncio
async def test_access_after_exit_denied(synthesizer: Synthesizer) -> None:
    async with synthesizer:
        pass
    with pytest.raises(ValueError, match="^The `Synthesizer` is closed$"):
        _ = synthesizer.metas()


@pytest_asyncio.fixture
async def synthesizer(onnxruntime: Onnxruntime, open_jtalk: OpenJtalk) -> Synthesizer:
    return Synthesizer(onnxruntime, open_jtalk)


@pytest_asyncio.fixture(scope="function")
async def onnxruntime() -> Onnxruntime:
    return await Onnxruntime.load_once(filename=conftest.onnxruntime_filename)


@pytest_asyncio.fixture(scope="function")
async def open_jtalk() -> OpenJtalk:
    return await OpenJtalk.new(conftest.open_jtalk_dic_dir)
