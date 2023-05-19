import dataclasses
import json
import logging
from argparse import ArgumentParser
from pathlib import Path
from typing import Tuple

import voicevox_core
from voicevox_core import AccelerationMode, AudioQuery, VoicevoxCore


def main() -> None:
    logging.basicConfig(format="[%(levelname)s] %(name)s: %(message)s")
    logger = logging.getLogger(__name__)
    logger.setLevel("DEBUG")
    logging.getLogger("voicevox_core_python_api").setLevel("DEBUG")
    logging.getLogger("voicevox_core").setLevel("DEBUG")

    (acceleration_mode, open_jtalk_dict_dir, text, out, speaker_id) = parse_args()

    logger.debug("%s", f"{voicevox_core.METAS=}")
    logger.debug("%s", f"{voicevox_core.SUPPORTED_DEVICES=}")

    logger.info("%s", f"Initializing ({acceleration_mode=}, {open_jtalk_dict_dir=})")
    core = VoicevoxCore(
        acceleration_mode=acceleration_mode, open_jtalk_dict_dir=open_jtalk_dict_dir
    )

    logger.debug("%s", f"{core.is_gpu_mode=}")

    logger.info("%s", f"Loading model {speaker_id}")
    core.load_model(speaker_id)

    logger.debug("%s", f"{core.is_model_loaded(0)=}")

    logger.info("%s", f"Creating an AudioQuery from {text!r}")
    audio_query = core.audio_query(text, speaker_id)

    logger.info("%s", f"Synthesizing with {display_as_json(audio_query)}")
    wav = core.synthesis(audio_query, speaker_id)

    out.write_bytes(wav)
    logger.info("%s", f"Wrote `{out}`")


def parse_args() -> Tuple[AccelerationMode, Path, str, Path, int]:
    argparser = ArgumentParser()
    argparser.add_argument(
        "--mode",
        default="AUTO",
        type=AccelerationMode,
        help='モード ("AUTO", "CPU", "GPU")',
    )
    argparser.add_argument(
        "--dict-dir",
        default="./open_jtalk_dic_utf_8-1.11",
        type=Path,
        help="Open JTalkの辞書ディレクトリ",
    )
    argparser.add_argument(
        "--text",
        default="この音声は、ボイスボックスを使用して、出力されています。",
        help="読み上げさせたい文章",
    )
    argparser.add_argument(
        "--out",
        default="./output.wav",
        type=Path,
        help="出力wavファイルのパス",
    )
    argparser.add_argument(
        "--speeker-id",
        default=0,
        type=int,
        help="話者IDを指定",
    )
    args = argparser.parse_args()
    return (args.mode, args.dict_dir, args.text, args.out, args.speeker_id)


def display_as_json(audio_query: AudioQuery) -> str:
    return json.dumps(dataclasses.asdict(audio_query), ensure_ascii=False)


if __name__ == "__main__":
    main()
