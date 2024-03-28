"""
モーフィング機能をテストする。

``test_asyncio_morph`` と対になる。
"""

from typing import Dict

import conftest
import pytest
from voicevox_core import SpeakerFeatureError, StyleId, StyleNotFoundError
from voicevox_core.blocking import OpenJtalk, Synthesizer, VoiceModel


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
def test_morph(
    synthesizer: Synthesizer, base: StyleId, targets: Dict[StyleId, bool]
) -> None:
    TEXT = "こんにちは"
    MORPH_RATE = 0.5

    query = synthesizer.audio_query(TEXT, base)

    for target, should_success in targets.items():
        is_morphable = synthesizer.morphable_targets(base)[target].is_morphable
        assert is_morphable == should_success

        if should_success:
            # TODO: スナップショットテストをやる
            synthesizer.synthesis_morphing(query, base, target, MORPH_RATE)
        else:
            with pytest.raises(
                SpeakerFeatureError,
                match=(
                    r"^`dummy[1-3]` \([0-9a-f-]{36}\)は以下の機能を持ちません: "
                    r"`dummy[1-3]` \([0-9a-f-]{36}\)に対するモーフィング$"
                ),
            ):
                synthesizer.synthesis_morphing(query, base, target, MORPH_RATE)


def test_morphable_targets_denies_unknown_style(synthesizer: Synthesizer) -> None:
    STYLE_ID = StyleId(9999)

    with pytest.raises(
        StyleNotFoundError,
        match=f"^'`{STYLE_ID}`に対するスタイルが見つかりませんでした。音声モデルが読み込まれていないか、読み込みが解除されています'$",
    ):
        synthesizer.morphable_targets(STYLE_ID)


@pytest.fixture
def synthesizer(open_jtalk: OpenJtalk, model: VoiceModel) -> Synthesizer:
    synthesizer = Synthesizer(open_jtalk)
    synthesizer.load_voice_model(model)
    return synthesizer


@pytest.fixture(scope="session")
def open_jtalk() -> OpenJtalk:
    return OpenJtalk(conftest.open_jtalk_dic_dir)


@pytest.fixture(scope="session")
def model() -> VoiceModel:
    return VoiceModel.from_path(conftest.model_dir)
