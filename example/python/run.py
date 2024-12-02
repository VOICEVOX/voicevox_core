import dataclasses
import json
import logging
from argparse import ArgumentParser
from pathlib import Path

from voicevox_core import AccelerationMode, AudioQuery, wav_from_s16le
from voicevox_core.blocking import Onnxruntime, OpenJtalk, Synthesizer, VoiceModelFile


@dataclasses.dataclass
class Args:
    mode: AccelerationMode
    vvm: Path
    onnxruntime: str
    dict_dir: Path
    text: str
    out: Path
    style_id: int
    streaming: bool

    @staticmethod
    def parse_args() -> "Args":
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
        argparser.add_argument(
            "--streaming",
            action="store_true",
            help="ストリーミング生成",
        )
        args = argparser.parse_args()
        return Args(
            args.mode,
            args.vvm,
            args.onnxruntime,
            args.dict_dir,
            args.text,
            args.out,
            args.style_id,
            args.streaming,
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
        onnxruntime, OpenJtalk(args.dict_dir), acceleration_mode=args.mode
    )

    logger.debug("%s", f"{synthesizer.metas=}")
    logger.debug("%s", f"{synthesizer.is_gpu_mode=}")

    logger.info("%s", f"Loading `{args.vvm}`")
    with VoiceModelFile.open(args.vvm) as model:
        synthesizer.load_voice_model(model)

    logger.info("%s", f"Creating an AudioQuery from {args.text!r}")
    audio_query = synthesizer.create_audio_query(args.text, args.style_id)

    logger.info("%s", f"Synthesizing with {display_as_json(audio_query)}")
    if args.streaming:
        logger.info("%s", "In streaming mode")
        chunk_sec = 1.0
        audio_feature = synthesizer.precompute_render(audio_query, args.style_id)
        chunk_frames = int(audio_feature.frame_rate * chunk_sec)
        pcm = b""
        for i in range(0, audio_feature.frame_length, chunk_frames):
            logger.info("%s", f"{i/audio_feature.frame_length:.2%}")
            pcm += synthesizer.render(
                audio_feature, i, min(i + chunk_frames, audio_feature.frame_length)
            )
        logger.info("%s", f"100%")
        wav = wav_from_s16le(
            pcm, audio_query.output_sampling_rate, audio_query.output_stereo
        )

    else:
        wav = synthesizer.synthesis(audio_query, args.style_id)

    args.out.write_bytes(wav)
    logger.info("%s", f"Wrote `{args.out}`")


def display_as_json(audio_query: AudioQuery) -> str:
    return json.dumps(dataclasses.asdict(audio_query), ensure_ascii=False)


if __name__ == "__main__":
    main()
