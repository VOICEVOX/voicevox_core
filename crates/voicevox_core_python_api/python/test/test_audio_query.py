import dataclasses
import json
import textwrap

import pytest
from voicevox_core import AudioQuery


def test_accept_json_without_optional_fields() -> None:
    AudioQuery(
        **json.loads(
            textwrap.dedent(
                """\
                {
                  "accent_phrases": [
                    {
                      "moras": [
                        {
                          "text": "ア",
                          "vowel": "a",
                          "vowel_length": 0.0,
                          "pitch": 0.0
                        }
                      ],
                      "accent": 1
                    }
                  ],
                  "speedScale": 1.0,
                  "pitchScale": 0.0,
                  "intonationScale": 1.0,
                  "volumeScale": 1.0,
                  "prePhonemeLength": 0.1,
                  "postPhonemeLength": 0.1,
                  "outputSamplingRate": 24000,
                  "outputStereo": false
                }
                """,
            )
        )
    )


def test_dumps() -> None:
    BEFORE = textwrap.dedent(
        """\
        {
          "accent_phrases": [],
          "speedScale": 1.0,
          "pitchScale": 0.0,
          "intonationScale": 1.0,
          "volumeScale": 1.0,
          "prePhonemeLength": 0.1,
          "postPhonemeLength": 0.1,
          "outputSamplingRate": 24000,
          "outputStereo": false,
          "pauseLength": null,
          "pauseLengthScale": 1.0,
          "kana": ""
        }""",
    )

    after = json.dumps(dataclasses.asdict(AudioQuery(**json.loads(BEFORE))), indent=2)
    assert BEFORE == after


# あまり保証したくない性質ではあるが、`dataclasses.asdict`に必要
def test_getattr() -> None:
    query = AudioQuery(
        **json.loads(
            textwrap.dedent(
                """\
                {
                  "accent_phrases": [],
                  "speedScale": 1.0,
                  "pitchScale": 0.0,
                  "intonationScale": 1.0,
                  "volumeScale": 1.0,
                  "prePhonemeLength": 0.1,
                  "postPhonemeLength": 0.1,
                  "outputSamplingRate": 24000,
                  "outputStereo": false
                }
                """,
            )
        )
    )

    assert getattr(query, "speedScale") is query.speed_scale
    assert getattr(query, "pitchScale") is query.pitch_scale
    assert getattr(query, "intonationScale") is query.intonation_scale
    assert getattr(query, "volumeScale") is query.volume_scale
    assert getattr(query, "prePhonemeLength") is query.pre_phoneme_length
    assert getattr(query, "postPhonemeLength") is query.post_phoneme_length
    assert getattr(query, "outputSamplingRate") is query.output_sampling_rate
    assert getattr(query, "outputStereo") is query.output_stereo
    assert getattr(query, "pauseLength") is query.pause_length
    assert getattr(query, "pauseLengthScale") is query.pause_length_scale

    with pytest.raises(
        AttributeError, match="^'AudioQuery' has no attribute 'nonexisting_name'$"
    ):
        getattr(query, "nonexisting_name")
