import dataclasses
import json
import logging
from argparse import ArgumentParser
from pathlib import Path
from typing import Tuple

import voicevox_core
from voicevox_core import AccelerationMode, AudioQuery, VoicevoxCore

SPEAKER_ID = 0


def main() -> None:
    logging.basicConfig(
        format="[%(levelname)s] %(filename)s: %(message)s", level="DEBUG"
    )
    logger = logging.getLogger(__name__)

    (acceleration_mode, open_jtalk_dict_dir, text, out) = parse_args()

    logger.info("%s", f"{voicevox_core.METAS=}")
    logger.info("%s", f"{voicevox_core.SUPPORTED_DEVICES=}")

    logger.info("%s", f"Initializing ({acceleration_mode=}, {open_jtalk_dict_dir=})")
    core = VoicevoxCore(
        acceleration_mode=acceleration_mode, open_jtalk_dict_dir=open_jtalk_dict_dir
    )

    logger.info("%s", f"Loading model {SPEAKER_ID}")
    core.load_model(SPEAKER_ID)

    logger.debug("%s", f"{core.is_model_loaded(0)=}")

    logger.info("%s", f"Creating an AudioQuery from {text!r}")
    audio_query = core.audio_query(text, SPEAKER_ID)

    logger.info("%s", f"Synthesizing with {display_as_json(audio_query)}")
    wav = core.synthesis(audio_query, SPEAKER_ID)

    out.write_bytes(wav)
    logger.info("%s", f"Wrote `{out}`")


def parse_args() -> Tuple[AccelerationMode, Path, str, Path]:
    argparser = ArgumentParser()
    argparser.add_argument("--mode", type=AccelerationMode)
    argparser.add_argument("open_jtalk_dict_dir", type=Path)
    argparser.add_argument("text")
    argparser.add_argument("out", type=Path)
    args = argparser.parse_args()
    return (args.mode, args.open_jtalk_dict_dir, args.text, args.out)


def display_as_json(audio_query: AudioQuery) -> str:
    return json.dumps(dataclasses.asdict(audio_query), ensure_ascii=False)


if __name__ == "__main__":
    main()
