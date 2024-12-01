"""asyncio版のサンプルコードです。"""

import asyncio
import dataclasses
import json
import logging
from argparse import ArgumentParser
from pathlib import Path
from typing import Tuple

from voicevox_core import AccelerationMode, AudioQuery
from voicevox_core.asyncio import Onnxruntime, OpenJtalk, Synthesizer, VoiceModelFile


async def main() -> None:
    logging.basicConfig(format="[%(levelname)s] %(name)s: %(message)s")
    logger = logging.getLogger(__name__)
    logger.setLevel("DEBUG")
    logging.getLogger("voicevox_core_python_api").setLevel("DEBUG")
    logging.getLogger("voicevox_core").setLevel("DEBUG")

    (
        acceleration_mode,
        vvm_path,
        onnxruntime_filename,
        open_jtalk_dict_dir,
        text,
        out,
        style_id,
    ) = parse_args()

    logger.info("%s", f"Loading ONNX Runtime ({onnxruntime_filename=})")
    onnxruntime = await Onnxruntime.load_once(filename=onnxruntime_filename)

    logger.debug("%s", f"{onnxruntime.supported_devices()=}")

    logger.info("%s", f"Initializing ({acceleration_mode=}, {open_jtalk_dict_dir=})")
    synthesizer = Synthesizer(
        onnxruntime,
        await OpenJtalk.new(open_jtalk_dict_dir),
        acceleration_mode=acceleration_mode,
    )

    logger.debug("%s", f"{synthesizer.metas=}")
    logger.debug("%s", f"{synthesizer.is_gpu_mode=}")

    logger.info("%s", f"Loading `{vvm_path}`")
    async with await VoiceModelFile.open(vvm_path) as model:
        await synthesizer.load_voice_model(model)

    logger.info("%s", f"Creating an AudioQuery from {text!r}")
    audio_query = await synthesizer.create_audio_query(text, style_id)

    logger.info("%s", f"Synthesizing with {display_as_json(audio_query)}")
    wav = await synthesizer.synthesis(audio_query, style_id)

    out.write_bytes(wav)
    logger.info("%s", f"Wrote `{out}`")


def parse_args() -> Tuple[AccelerationMode, Path, str, Path, str, Path, int]:
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
        "--onnxruntime",
        default=Onnxruntime.LIB_VERSIONED_FILENAME,
        help="ONNX Runtimeのライブラリのfilename",
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
        "--style-id",
        default=0,
        type=int,
        help="話者IDを指定",
    )
    args = argparser.parse_args()
    # FIXME: 流石に多くなってきたので、`dataclass`化する
    return (
        args.mode,
        args.vvm,
        args.onnxruntime,
        args.dict_dir,
        args.text,
        args.out,
        args.style_id,
    )


def display_as_json(audio_query: AudioQuery) -> str:
    return json.dumps(dataclasses.asdict(audio_query), ensure_ascii=False)


if __name__ == "__main__":
    asyncio.run(main())
