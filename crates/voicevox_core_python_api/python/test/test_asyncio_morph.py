"""
モーフィング機能をテストする。

``test_blocking_morph`` と対になる。
"""

from typing import Dict

import conftest
import pytest
import pytest_asyncio
from voicevox_core import SpeakerFeatureError, StyleId, StyleNotFoundError
from voicevox_core.asyncio import OpenJtalk, Synthesizer, VoiceModel


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "base, targets",
    [
        (
            0,
            {
                0: False,
                1: False,
                302: False,
                303: False,
            },
        ),
        (
            1,
            {
                0: False,
                1: True,
                302: False,
                303: False,
            },
        ),
        (
            302,
            {
                0: False,
                1: False,
                302: True,
                303: True,
            },
        ),
        (
            303,
            {
                0: False,
                1: False,
                302: True,
                303: True,
            },
        ),
    ],
)
async def test_morph(
    synthesizer: Synthesizer, base: StyleId, targets: Dict[StyleId, bool]
) -> None:
    TEXT = "こんにちは"
    MORPH_RATE = 0.5

    query = await synthesizer.audio_query(TEXT, base)

    for target, should_success in targets.items():
        is_morphable = synthesizer.morphable_targets(base)[target].is_morphable
        assert is_morphable == should_success

        if should_success:
            # TODO: スナップショットテストをやる
            await synthesizer.synthesis_morphing(query, base, target, MORPH_RATE)
        else:
            with pytest.raises(
                SpeakerFeatureError,
                match=(
                    r"^`dummy[1-3]` \([0-9a-f-]{36}\)は以下の機能を持ちません: "
                    r"`dummy[1-3]` \([0-9a-f-]{36}\)に対するモーフィング$"
                ),
            ):
                await synthesizer.synthesis_morphing(query, base, target, MORPH_RATE)


def test_morphable_targets_denies_unknown_style(synthesizer: Synthesizer) -> None:
    STYLE_ID = StyleId(9999)

    with pytest.raises(
        StyleNotFoundError,
        match=rf"^'`{STYLE_ID}` \(\[talk\]\)に対するスタイルが見つかりませんでした。音声モデルが読み込まれていないか、読み込みが解除されています'$",
    ):
        synthesizer.morphable_targets(STYLE_ID)


@pytest_asyncio.fixture
async def synthesizer(open_jtalk: OpenJtalk, model: VoiceModel) -> Synthesizer:
    synthesizer = Synthesizer(open_jtalk)
    await synthesizer.load_voice_model(model)
    return synthesizer


@pytest_asyncio.fixture
async def open_jtalk() -> OpenJtalk:
    return await OpenJtalk.new(conftest.open_jtalk_dic_dir)


@pytest_asyncio.fixture
async def model() -> VoiceModel:
    return await VoiceModel.from_path(conftest.model_dir)
