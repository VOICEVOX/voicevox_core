import asyncio
import dataclasses
import json
import logging
from argparse import ArgumentParser
from pathlib import Path
from typing import Tuple

import voicevox_core
from voicevox_core import (
    AccelerationMode,
    AudioQuery,
    OpenJtalk,
    Synthesizer,
    VoiceModel,
)

SPEAKER_ID = 0


async def main() -> None:
    logging.basicConfig(format="[%(levelname)s] %(name)s: %(message)s")
    logger = logging.getLogger(__name__)
    logger.setLevel("DEBUG")
    logging.getLogger("voicevox_core_python_api").setLevel("DEBUG")
    logging.getLogger("voicevox_core").setLevel("DEBUG")

    (
        acceleration_mode,
        vvm_path,
        open_jtalk_dict_dir,
        text,
        out,
        speaker_id,
    ) = parse_args()

    logger.debug("%s", f"{voicevox_core.supported_devices()=}")

    logger.info("%s", f"Initializing ({acceleration_mode=}, {open_jtalk_dict_dir=})")
    synthesizer = await Synthesizer.new_with_initialize(
        OpenJtalk(open_jtalk_dict_dir), acceleration_mode=acceleration_mode
    )

    logger.debug("%s", f"{synthesizer.metas=}")
    logger.debug("%s", f"{synthesizer.is_gpu_mode=}")

    logger.info("%s", f"Loading `{vvm_path}`")
    model = await VoiceModel.from_path(vvm_path)
    await synthesizer.load_voice_model(model)

    logger.info("%s", f"Creating an AudioQuery from {text!r}")
    audio_query = await synthesizer.audio_query(text, speaker_id)

    logger.info("%s", f"Synthesizing with {display_as_json(audio_query)}")
    wav = await synthesizer.synthesis(audio_query, speaker_id)

    out.write_bytes(wav)
    logger.info("%s", f"Wrote `{out}`")


def parse_args() -> Tuple[AccelerationMode, Path, Path, str, Path, int]:
    argparser = ArgumentParser()
    argparser.add_argument(
        "--mode",
        default="AUTO",
        type=AccelerationMode,
        help='モード ("AUTO", "CPU", "GPU")',
    )
    argparser.add_argument(
        "vvm",
        type=Path,
        help="vvmファイルへのパス",
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
    return (args.mode, args.vvm, args.dict_dir, args.text, args.out, args.speeker_id)


def display_as_json(audio_query: AudioQuery) -> str:
    return json.dumps(dataclasses.asdict(audio_query), ensure_ascii=False)


if __name__ == "__main__":
    asyncio.run(main())
