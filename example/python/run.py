import argparse
from typing import Optional

import core
import soundfile

from forwarder import Forwarder


def run(
    use_gpu: bool,
    text: str,
    speaker_id: int,
    f0_speaker_id: Optional[int],
    f0_correct: float,
    root_dir_path: str,
    cpu_num_threads: int
) -> None:
    # コアの初期化
    core.initialize(root_dir_path, use_gpu, cpu_num_threads)

    # 音声合成処理モジュールの初期化
    forwarder = Forwarder(
        yukarin_s_forwarder=core.yukarin_s_forward,
        yukarin_sa_forwarder=core.yukarin_sa_forward,
        decode_forwarder=core.decode_forward,
    )

    # 音声合成
    wave = forwarder.forward(
        text=text,
        speaker_id=speaker_id,
        f0_speaker_id=f0_speaker_id if f0_speaker_id is not None else speaker_id,
        f0_correct=f0_correct,
    )

    # 保存
    soundfile.write(f"{text}-{speaker_id}.wav", data=wave, samplerate=24000)

    core.finalize()


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--use_gpu", action="store_true")
    parser.add_argument("--text", required=True)
    parser.add_argument("--speaker_id", type=int, required=True)
    parser.add_argument("--f0_speaker_id", type=int)
    parser.add_argument("--f0_correct", type=float, default=0)
    parser.add_argument("--root_dir_path", type=str, default="./")
    parser.add_argument("--cpu_num_threads", type=int, default=0)
    run(**vars(parser.parse_args()))
