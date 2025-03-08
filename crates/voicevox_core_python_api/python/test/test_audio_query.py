import dataclasses
import textwrap

import pytest
from voicevox_core import AudioQuery


def test_accept_json_without_optional_fields() -> None:
    from_json(
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
          "kana": ""
        }""",
    )

    after = to_json(from_json(BEFORE))
    assert BEFORE.replace("\n", "").replace(" ", "") == after


def from_json(json: str) -> AudioQuery:
    return getattr(AudioQuery, "_AudioQuery__from_json")(json)


def to_json(audio_query: AudioQuery) -> str:
    return getattr(audio_query, "_AudioQuery__to_json")()
