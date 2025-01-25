import json
import textwrap

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
