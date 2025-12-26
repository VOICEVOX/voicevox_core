#!/usr/bin/env python

"""asyncio版の歌唱音声合成を行うサンプルコードです。"""

import asyncio
import dataclasses
import logging
import multiprocessing
from argparse import ArgumentParser
from pathlib import Path

from voicevox_core import AccelerationMode, Note, Score
from voicevox_core.asyncio import Onnxruntime, OpenJtalk, Synthesizer, VoiceModelFile


@dataclasses.dataclass
class Args:
    mode: AccelerationMode
    vvm: Path
    onnxruntime: str
    dict_dir: Path
    out: Path
    singing_teacher: int
    singer: int

    @staticmethod
    def parse_args() -> "Args":
        argparser = ArgumentParser()
        argparser.add_argument(
            "--mode",
            default="AUTO",
            choices=("AUTO", "CPU", "GPU"),
            help="モード",
        )
        argparser.add_argument(
            "vvm",
            type=Path,
            help="vvmファイルへのパス",
        )
        argparser.add_argument(
            "--onnxruntime",
            default=f"./onnxruntime/lib/{Onnxruntime.LIB_VERSIONED_FILENAME}",
            help="ONNX Runtimeのライブラリのfilename",
        )
        argparser.add_argument(
            "--dict-dir",
            default="./dict/open_jtalk_dic_utf_8-1.11",
            type=Path,
            help="Open JTalkの辞書ディレクトリ",
        )
        argparser.add_argument(
            "--out",
            default="./output.wav",
            type=Path,
            help="出力wavファイルのパス",
        )
        argparser.add_argument(
            "--singing-teacher",
            default=6000,
            type=int,
            help="",
        )
        argparser.add_argument(
            "--singer",
            default=3000,
            type=int,
            help="",
        )
        args = argparser.parse_args()
        return Args(
            args.mode,
            args.vvm,
            args.onnxruntime,
            args.dict_dir,
            args.out,
            args.singing_teacher,
            args.singer,
        )


async def main() -> None:
    logging.basicConfig(format="[%(levelname)s] %(name)s: %(message)s")
    logger = logging.getLogger(__name__)
    logger.setLevel("DEBUG")
    logging.getLogger("voicevox_core_python_api").setLevel("DEBUG")
    logging.getLogger("voicevox_core").setLevel("DEBUG")

    args = Args.parse_args()

    logger.info("%s", f"Loading ONNX Runtime ({args.onnxruntime=})")
    onnxruntime = await Onnxruntime.load_once(filename=args.onnxruntime)

    logger.debug("%s", f"{onnxruntime.supported_devices()=}")

    logger.info("%s", f"Initializing ({args.mode=}, {args.dict_dir=})")
    synthesizer = Synthesizer(
        onnxruntime,
        await OpenJtalk.new(args.dict_dir),
        acceleration_mode=args.mode,
        cpu_num_threads=max(
            multiprocessing.cpu_count(), 2
        ),  # https://github.com/VOICEVOX/voicevox_core/issues/888
    )
    logger.debug("%s", f"{synthesizer.is_gpu_mode=}")

    logger.info("%s", f"Loading `{args.vvm}`")
    async with await VoiceModelFile.open(args.vvm) as model:
        await synthesizer.load_voice_model(model)
    logger.debug("%s", f"{synthesizer.metas()=}")

    SCORE = Score(
        [
            Note(15, ""),
            Note(45, "ド", key=60),
            Note(45, "レ", key=62),
            Note(45, "ミ", key=64),
            Note(15, ""),
        ],
    )

    logger.info("%s", f"Creating an AudioQuery from {SCORE}")
    frame_audio_query = await synthesizer.create_sing_frame_audio_query(
        SCORE, args.singing_teacher
    )

    logger.info("%s", f"Synthesizing with {frame_audio_query}")
    wav = await synthesizer.frame_synthesis(frame_audio_query, args.singer)

    args.out.write_bytes(wav)
    logger.info("%s", f"Wrote `{args.out}`")


if __name__ == "__main__":
    asyncio.run(main())
