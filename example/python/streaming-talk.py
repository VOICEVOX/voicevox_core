#!/usr/bin/env python

"""リアルタイムテキスト音声合成を行うサンプルコードです。"""

import dataclasses
import logging
import multiprocessing
import operator
import time
from argparse import ArgumentParser
from pathlib import Path

from sounddevice import RawOutputStream
from voicevox_core import FRAME_RATE, AccelerationMode
from voicevox_core.blocking import Onnxruntime, OpenJtalk, Synthesizer, VoiceModelFile


@dataclasses.dataclass
class Args:
    mode: AccelerationMode
    vvm: Path
    onnxruntime: str
    dict_dir: Path
    text: str
    segment_length: float
    style_id: int

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
            default=f"./onnxruntime/lib/{Onnxruntime.LIB_RECOMMENDED_VERSIONED_FILENAME}",
            help="ONNX Runtimeのライブラリのfilename",
        )
        argparser.add_argument(
            "--dict-dir",
            default="./dict/open_jtalk_dic_utf_8-1.11",
            type=Path,
            help="Open JTalkの辞書ディレクトリ",
        )
        argparser.add_argument(
            "--text",
            default="この音声は、ボイスボックスを使用して、出力されています。",
            help="読み上げさせたい文章",
        )
        argparser.add_argument(
            "--segment-length",
            default=0.3,
            type=float,
            help="一度に合成する音声の長さ",
        )
        argparser.add_argument(
            "--style-id",
            default=0,
            type=int,
            help="話者IDを指定",
        )
        args = argparser.parse_args()
        return Args(
            args.mode,
            args.vvm,
            args.onnxruntime,
            args.dict_dir,
            args.text,
            args.segment_length,
            args.style_id,
        )


def main() -> None:
    logging.basicConfig(format="[%(levelname)s] %(name)s: %(message)s")
    logger = logging.getLogger(__name__)
    logger.setLevel("DEBUG")
    logging.getLogger("voicevox_core_python_api").setLevel("DEBUG")
    logging.getLogger("voicevox_core").setLevel("DEBUG")

    args = Args.parse_args()

    logger.info("%s", f"Loading ONNX Runtime ({args.onnxruntime=})")
    onnxruntime = Onnxruntime.load_once(filename=args.onnxruntime)

    logger.debug("%s", f"{onnxruntime.supported_devices()=}")

    logger.info("%s", f"Initializing ({args.mode=}, {args.dict_dir=})")
    synthesizer = Synthesizer(
        onnxruntime,
        OpenJtalk(args.dict_dir),
        acceleration_mode=args.mode,
        cpu_num_threads=max(
            multiprocessing.cpu_count(), 2
        ),  # https://github.com/VOICEVOX/voicevox_core/issues/888
    )
    logger.debug("%s", f"{synthesizer.is_gpu_mode=}")

    logger.info("%s", f"Loading `{args.vvm}`")
    with VoiceModelFile.open(args.vvm) as model:
        synthesizer.load_voice_model(model)
    logger.debug("%s", f"{synthesizer.metas()=}")

    logger.info("%s", f"Creating an AudioQuery from {args.text!r}")
    audio_query = synthesizer.create_audio_query(args.text, args.style_id)
    assert audio_query.output_sampling_rate == 24000

    logger.info("%s", f"Preparing the stream with {audio_query}")

    # TODO: specify `args.segment_length`
    stream = synthesizer.streaming_synthesis(audio_query, args.style_id)

    # wav_header = next(stream)
    _wav_header = next(stream)

    # import struct
    # import wave
    # from io import BytesIO
    #
    # with BytesIO(wav_header) as buf, wave.open(buf, "rb") as wav_header_:
    #     assert wav_header_.getnchannels() == (2 if audio_query.output_stereo else 1)
    #     assert wav_header_.getsampwidth() == struct.calcsize("h")
    #     assert wav_header_.getframerate() == 24000
    #     assert wav_header_.getnframes() == (
    #         # `audio_query.output_sampling_rate == 24000`の場合
    #         audio_query.frame_length()
    #         * int(24000 / FRAME_RATE)
    #     )
    #     assert wav_header_.getcomptype() == "NONE"
    # assert wav_header[-8:-4] == b"data"

    logger.info("Starting the real time synthesis")
    num_wrote_segments = 0
    num_total_segments = operator.length_hint(stream)

    with RawOutputStream(
        samplerate=audio_query.output_sampling_rate,
        channels=2 if audio_query.output_stereo else 1,
        dtype="int16",
        latency=args.segment_length + 0.1,
    ) as out:
        rendering_started = time.monotonic_ns()
        for segment in stream:
            underflowed = out.write(segment)
            if underflowed:
                logger.warning("Underrun occurred")
            num_wrote_segments += 1
            logger.info(
                "%s",
                "Appended a PCM segment to the buffer "
                f"({num_wrote_segments}/{num_total_segments})",
            )
        estimated_remaining_playback = (
            audio_query.frame_length() / FRAME_RATE
            - (time.monotonic_ns() - rendering_started) / 1e9
        )
        if estimated_remaining_playback < 0.0:
            logger.warning(
                "Synthesis exceeded the audio duration by %.3f seconds. "
                "Consider setting larger `--segment-length`",
                -estimated_remaining_playback,
            )
        time.sleep(max(0.0, estimated_remaining_playback + 0.1))


if __name__ == "__main__":
    main()
