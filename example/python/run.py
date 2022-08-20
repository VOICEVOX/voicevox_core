import argparse

import core


def run(
    use_gpu: bool,
    text: str,
    speaker_id: int,
    cpu_num_threads: int,
    openjtalk_dict: str
) -> None:
    # コアの初期化
    core.initialize(use_gpu, cpu_num_threads, load_all_models=False)

    # openjtalk辞書のロード
    core.voicevox_load_openjtalk_dict(openjtalk_dict)

    # 話者のロード
    core.load_model(speaker_id)

    # AudioQueryの生成
    audio_query = core.voicevox_audio_query(text, speaker_id)

    # 音声合成
    wavefmt = core.voicevox_synthesis(audio_query, speaker_id)

    # 保存
    with open(f"{text}-{speaker_id}.wav", "wb") as f:
        f.write(wavefmt)

    core.finalize()


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--use_gpu", action="store_true")
    parser.add_argument("--text", required=True)
    parser.add_argument("--speaker_id", type=int, required=True)
    parser.add_argument("--cpu_num_threads", type=int, default=0)
    parser.add_argument("--openjtalk_dict", type=str, default="voicevox_core/open_jtalk_dic_utf_8-1.11")
    run(**vars(parser.parse_args()))
