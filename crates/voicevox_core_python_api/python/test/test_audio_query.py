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
                  "speed_scale": 1.0,
                  "pitch_scale": 0.0,
                  "intonation_scale": 1.0,
                  "volume_scale": 1.0,
                  "pre_phoneme_length": 0.1,
                  "post_phoneme_length": 0.1,
                  "output_sampling_rate": 24000,
                  "output_stereo": false
                }
                """,
            )
        )
    )
