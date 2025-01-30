import dataclasses
import json
import textwrap

import pytest
from pydantic import TypeAdapter
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

    adapter = TypeAdapter(AudioQuery)
    query = adapter.validate_json(BEFORE)
    after = adapter.dump_json(query, indent=2, by_alias=True).decode()
    assert BEFORE == after
