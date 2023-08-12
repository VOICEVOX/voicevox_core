import conftest
import pytest
from voicevox_core import OpenJtalk, Synthesizer, VoicevoxError


@pytest.mark.asyncio
async def test_enter() -> None:
    open_jtalk = OpenJtalk(conftest.open_jtalk_dic_dir)

    with await Synthesizer.new_with_initialize(open_jtalk) as synthesizer:
        synthesizer.metas


@pytest.mark.asyncio
async def test_access_after_close_denied() -> None:
    open_jtalk = OpenJtalk(conftest.open_jtalk_dic_dir)

    synthesizer = await Synthesizer.new_with_initialize(open_jtalk)
    synthesizer.close()
    with pytest.raises(VoicevoxError, match="^The `Synthesizer` is closed$"):
        synthesizer.metas


@pytest.mark.asyncio
async def test_access_after_exit_denied() -> None:
    open_jtalk = OpenJtalk(conftest.open_jtalk_dic_dir)

    with await Synthesizer.new_with_initialize(open_jtalk) as synthesizer:
        pass
    with pytest.raises(VoicevoxError, match="^The `Synthesizer` is closed$"):
        synthesizer.metas
