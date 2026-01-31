# Python サンプルコード (PyO3 によるバインディング経由)

voicevox_core ライブラリ の Python バインディングを使った音声合成のサンプルコードです。  
`pip install`で導入することができます。

## 準備

1. wheel パッケージのインストールをします。

`[バージョン]`の部分は適宜書き換えてください。

```console
❯ pip install https://github.com/VOICEVOX/voicevox_core/releases/download/[バージョン]/voicevox_core-[バージョン]+cpu-cp310-abi3-linux_x86_64.whl
```

cpu-cp310-abi3-linux_x86_64 のところはアーキテクチャや OS によって適宜読み替えてください。
https://github.com/VOICEVOX/voicevox_core/releases/latest

2. ダウンローダーを使って環境構築します。

linux/mac の場合

download-linux-x64 のところはアーキテクチャや OS によって適宜読み替えてください。
https://github.com/VOICEVOX/voicevox_core/releases/latest#%E3%83%80%E3%82%A6%E3%83%B3%E3%83%AD%E3%83%BC%E3%83%80

```console
❯ binary=download-linux-x64
❯ curl -sSfL https://github.com/VOICEVOX/voicevox_core/releases/latest/download/${binary} -o download
❯ chmod +x download
❯ ./download -o ./example/python --exclude c-api
```

windows の場合

```console
❯ Invoke-WebRequest https://github.com/VOICEVOX/voicevox_core/releases/latest/download/download-windows-x64.exe -OutFile ./download.exe
❯ ./download -o ./example/python --exclude c-api
```

TODO:

- Python インタプリタ ≧3.10 + venv

## テキスト音声合成の実行

talk.py もしくは talk-asyncio.py を実行します。 Open JTalk 辞書ディレクトリ、読み上げさせたい文章、出力 wav ファイルのパスをオプションで指定することができます。

```console
❯ python ./talk.py -h
usage: talk.py [-h] [--mode MODE] [--dict-dir DICT_DIR] [--text TEXT] [--out OUT] [--style-id STYLE_ID] vvm

positional arguments:
  vvm                   vvmファイルへのパス

optional arguments:
  -h, --help            show this help message and exit
  --mode MODE           モード ("AUTO", "CPU", "GPU")
  --dict-dir DICT_DIR   Open JTalkの辞書ディレクトリ
  --text TEXT           読み上げさせたい文章
  --out OUT             出力wavファイルのパス
  --style-id STYLE_ID
                        話者IDを指定
```

## テキスト音声合成の実行例

```console
❯ python ./talk.py ./models/vvms/0.vvm
[INFO] __main__: Loading ONNX Runtime (args.onnxruntime='./onnxruntime/lib/libvoicevox_onnxruntime.so.1.17.3')
[DEBUG] __main__: onnxruntime.supported_devices()=SupportedDevices(cpu=True, cuda=True, dml=False)
[INFO] __main__: Initializing (args.mode=<AccelerationMode.AUTO: 'AUTO'>, args.dict_dir=PosixPath('dict/open_jtalk_dic_utf_8-1.11'))
[INFO] voicevox_core.synthesizer: GPUをテストします:
[INFO] voicevox_core.synthesizer:   * CUDA (device_id=0): OK
[INFO] voicevox_core.synthesizer:   * DirectML (device_id=0): 現在ロードされているONNX Runtimeでは利用できません
[INFO] voicevox_core.synthesizer: CUDA (device_id=0)を利用します
[DEBUG] __main__: synthesizer.is_gpu_mode=True
[INFO] __main__: Loading `models/vvms/0.vvm`
[WARNING] ort.environment: Some nodes were not assigned to the preferred execution providers which may or may not have an negative impact on performance. e.g. ORT explicitly assigns shape related ops to CPU to improve perf.
[DEBUG] voicevox_core_python_api: Closing a VoiceModelFile
[DEBUG] __main__: synthesizer.metas()=[CharacterMeta(name='四国めたん', styles=[StyleMeta(name='ノーマル', id=2, type='talk', order=None), StyleMeta(name=' あまあま', id=0, type='talk', order=None), StyleMeta(name='ツンツン', id=6, type='talk', order=None), StyleMeta(name='セクシー', id=4, type='talk', order=None)], speaker_uuid='7ffcb7ce-00ec-4bdc-82cd-45a8889e43ff', version='0.1.0', order=None), CharacterMeta(name='ずんだもん', styles=[StyleMeta(name='ノーマル', id=3, type='talk', order=None), StyleMeta(name='あまあま', id=1, type='talk', order=None), StyleMeta(name='ツンツン', id=7, type='talk', order=None), StyleMeta(name='セクシー', id=5, type='talk', order=None)], speaker_uuid='388f246b-8c41-4ac1-8e2d-5d79f3ff56d9', version='0.1.0', order=None), CharacterMeta(name='春日部つむぎ', styles=[StyleMeta(name='ノーマル', id=8, type='talk', order=None)], speaker_uuid='35b2c544-660e-401e-b503-0e14c635303a', version='0.1.0', order=None), CharacterMeta(name='雨晴はう', styles=[StyleMeta(name='ノーマル', id=10, type='talk', order=None)], speaker_uuid='3474ee95-c274-47f9-aa1a-8322163d96f1', version='0.1.0', order=None)]
[INFO] __main__: Creating an AudioQuery from 'この音声は、ボイスボックスを使用して、出力されています。'
[INFO] __main__: Synthesizing with AudioQuery(accent_phrases=[…], speed_scale=1.0, pitch_scale=0.0, intonation_scale=1.0, volume_scale=1.0, pre_phoneme_length=0.10000000149011612, post_phoneme_length=0.10000000149011612, output_sampling_rate=24000, output_stereo=False, kana="コノ'/オ'ンセエワ、ボイスボ'ッ_クスオ/シヨオ'/_シテ'、_シュツ'リョ_ク/サレテ'/イマ'_ス")
[INFO] __main__: Wrote `output.wav`
```

正常に実行されれば音声合成の結果である wav ファイルが生成されます。
この例の場合、`"この音声は、ボイスボックスを使用して、出力されています。"`という読み上げの wav ファイルが output.wav という名前で生成されます。

## 歌唱音声合成の実行

song.py もしくは song-asyncio.py を実行します。

```console
❯ python ./song.py -h
usage: song.py [-h] [--mode {AUTO,CPU,GPU}] [--onnxruntime ONNXRUNTIME] [--dict-dir DICT_DIR] [--out OUT] [--singing-teacher SINGING_TEACHER] [--singer SINGER] vvm

positional arguments:
  vvm                   vvmファイルへのパス

options:
  -h, --help            show this help message and exit
  --mode {AUTO,CPU,GPU}
                        モード
  --onnxruntime ONNXRUNTIME
                        ONNX Runtimeのライブラリのfilename
  --dict-dir DICT_DIR   Open JTalkの辞書ディレクトリ
  --out OUT             出力wavファイルのパス
  --singing-teacher SINGING_TEACHER
                        `type`が`singing_teacher`または`sing`であるスタイルID
  --singer SINGER       `type`が`frame_decode`であるスタイルID
```

## 歌唱音声合成の実行例

```console
❯ python ./song.py ./models/vvms/s0.vvm --singer 3002
[INFO] __main__: Loading ONNX Runtime (args.onnxruntime='./onnxruntime/lib/libvoicevox_onnxruntime.so.1.17.3')
[DEBUG] __main__: onnxruntime.supported_devices()=SupportedDevices(cpu=True, cuda=True, dml=False)
[INFO] __main__: Initializing (args.mode='AUTO', args.dict_dir=PosixPath('dict/open_jtalk_dic_utf_8-1.11'))
[INFO] voicevox_core.synthesizer: GPUをテストします:
[INFO] voicevox_core.synthesizer:   * CUDA (device_id=0): OK
[INFO] voicevox_core.synthesizer:   * DirectML (device_id=0): 現在ロードされているONNX Runtimeでは利用できません
[INFO] voicevox_core.synthesizer: CUDA (device_id=0)を利用します
[DEBUG] __main__: synthesizer.is_gpu_mode=True
[INFO] __main__: Loading `./models/vvms/s0.vvm`
[WARNING] ort.logging: 9 Memcpy nodes are added to the graph torch_jit for CUDAExecutionProvider. It might have negative impact on performance (including unable to run CUDA graph). Set session_options.log_severity_level=1 to see the detail logs before this message.
[WARNING] ort.logging: Some nodes were not assigned to the preferred execution providers which may or may not have an negative impact on performance. e.g. ORT explicitly assigns shape related ops to CPU to improve perf.
[DEBUG] voicevox_core_python_api: Closing a VoiceModelFile
[DEBUG] __main__: synthesizer.metas()=[CharacterMeta(name='四国めたん', styles=[StyleMeta(name='ノーマル', id=3002, type='frame_decode', order=0), StyleMeta(name='あまあま', id=3000, type='frame_decode', order=1), StyleMeta(name='ツンツン', id=3006, type='frame_decode', order=2), StyleMeta(name='セクシー', id=3004, type='frame_decode', order=3), StyleMeta(name='ヒソヒソ', id=3037, type='frame_decode', order=4)], speaker_uuid='7ffcb7ce-00ec-4bdc-82cd-45a8889e43ff', version='0.16.1', order=0), CharacterMeta(name='ずんだもん', styles=[StyleMeta(name='ノーマル', id=3003, type='frame_decode', order=0), StyleMeta(name='あ まあま', id=3001, type='frame_decode', order=1), StyleMeta(name='ツンツン', id=3007, type='frame_decode', order=2), StyleMeta(name='セクシー', id=3005, type='frame_decode', order=3), StyleMeta(name='ヒソヒソ', id=3038, type='frame_decode', order=4), StyleMeta(name='ヘロヘロ', id=3075, type='frame_decode', order=5), StyleMeta(name='なみだめ', id=3076, type='frame_decode', order=6)], speaker_uuid='388f246b-8c41-4ac1-8e2d-5d79f3ff56d9', version='0.16.1', order=1), CharacterMeta(name='春日部つむぎ', styles=[StyleMeta(name='ノーマル', id=3008, type='frame_decode', order=0)], speaker_uuid='35b2c544-660e-401e-b503-0e14c635303a', version='0.16.1', order=2), CharacterMeta(name='雨晴はう', styles=[StyleMeta(name='ノーマル', id=3010, type='frame_decode', order=0)], speaker_uuid='3474ee95-c274-47f9-aa1a-8322163d96f1', version='0.16.1', order=3), CharacterMeta(name='波音リツ', styles=[StyleMeta(name='ノーマル', id=6000, type='sing', order=0), StyleMeta(name='ノーマル', id=3009, type='frame_decode', order=1), StyleMeta(name='クイーン', id=3065, type='frame_decode', order=2)], speaker_uuid='b1a81618-b27b-40d2-b0ea-27a9ad408c4b', version='0.16.1', order=4), CharacterMeta(name='玄野武宏', styles=[StyleMeta(name='ノーマル', id=3011, type='frame_decode', order=0), StyleMeta(name='喜び', id=3039, type='frame_decode', order=1), StyleMeta(name='ツンギレ', id=3040, type='frame_decode', order=2), StyleMeta(name='悲しみ', id=3041, type='frame_decode', order=3)], speaker_uuid='c30dc15a-0992-4f8d-8bb8-ad3b314e6a6f', version='0.16.1', order=5), CharacterMeta(name='白上虎太郎', styles=[StyleMeta(name='ふつう', id=3012, type='frame_decode', order=0), StyleMeta(name='わーい', id=3032, type='frame_decode', order=1), StyleMeta(name='びくびく', id=3033, type='frame_decode', order=2), StyleMeta(name='おこ', id=3034, type='frame_decode', order=3), StyleMeta(name='びえーん', id=3035, type='frame_decode', order=4)], speaker_uuid='e5020595-5c5d-4e87-b849-270a518d0dcf', version='0.16.1', order=6), CharacterMeta(name='青山龍星', styles=[StyleMeta(name='ノーマル', id=3013, type='frame_decode', order=0), StyleMeta(name='熱血', id=3081, type='frame_decode', order=1), StyleMeta(name='不機嫌', id=3082, type='frame_decode', order=2), StyleMeta(name='喜び', id=3083, type='frame_decode', order=3), StyleMeta(name='しっとり', id=3084, type='frame_decode', order=4), StyleMeta(name='かなしみ', id=3085, type='frame_decode', order=5)], speaker_uuid='4f51116a-d9ee-4516-925d-21f183e2afad', version='0.16.1', order=7), CharacterMeta(name='冥鳴ひまり', styles=[StyleMeta(name='ノーマル', id=3014, type='frame_decode', order=0)], speaker_uuid='8eaad775-3119-417e-8cf4-2a10bfd592c8', version='0.16.1', order=8), CharacterMeta(name='九州そら', styles=[StyleMeta(name='ノーマル', id=3016, type='frame_decode', order=0), StyleMeta(name='あまあま', id=3015, type='frame_decode', order=1), StyleMeta(name='ツンツン', id=3018, type='frame_decode', order=2), StyleMeta(name='セクシー', id=3017, type='frame_decode', order=3)], speaker_uuid='481fb609-6446-4870-9f46-90c4dd623403', version='0.16.1', order=9), CharacterMeta(name='もち子さん', styles=[StyleMeta(name='ノーマル', id=3020, type='frame_decode', order=0), StyleMeta(name='セクシー／あん子', id=3066, type='frame_decode', order=1), StyleMeta(name='泣き', id=3077, type='frame_decode', order=2), StyleMeta(name='怒り', id=3078, type='frame_decode', order=3), StyleMeta(name='喜び', id=3079, type='frame_decode', order=4), StyleMeta(name='のんびり', id=3080, type='frame_decode', order=5)], speaker_uuid='9f3ee141-26ad-437e-97bd-d22298d02ad2', version='0.16.1', order=10), CharacterMeta(name='剣崎雌雄', styles=[StyleMeta(name='ノーマル', id=3021, type='frame_decode', order=0)], speaker_uuid='1a17ca16-7ee5-4ea5-b191-2f02ace24d21', version='0.16.1', order=11), CharacterMeta(name='WhiteCUL', styles=[StyleMeta(name='ノーマル', id=3023, type='frame_decode', order=0), StyleMeta(name='たのしい', id=3024, type='frame_decode', order=1), StyleMeta(name='かなしい', id=3025, type='frame_decode', order=2), StyleMeta(name='びえーん', id=3026, type='frame_decode', order=3)], speaker_uuid='67d5d8da-acd7-4207-bb10-b5542d3a663b', version='0.16.1', order=12), CharacterMeta(name='後鬼', styles=[StyleMeta(name='人間ver.', id=3027, type='frame_decode', order=0), StyleMeta(name='ぬいぐるみver.', id=3028, type='frame_decode', order=1)], speaker_uuid='0f56c2f2-644c-49c9-8989-94e11f7129d0', version='0.16.1', order=13), CharacterMeta(name='No.7', styles=[StyleMeta(name='ノーマル', id=3029, type='frame_decode', order=0), StyleMeta(name='アナウンス', id=3030, type='frame_decode', order=1), StyleMeta(name='読み聞かせ', id=3031, type='frame_decode', order=2)], speaker_uuid='044830d2-f23b-44d6-ac0d-b5d733caa900', version='0.16.1', order=14), CharacterMeta(name='ちび式じい', styles=[StyleMeta(name='ノーマル', id=3042, type='frame_decode', order=0)], speaker_uuid='468b8e94-9da4-4f7a-8715-a22a48844f9e', version='0.16.1', order=15), CharacterMeta(name='櫻歌ミコ', styles=[StyleMeta(name='ノーマル', id=3043, type='frame_decode', order=0), StyleMeta(name='第二形態', id=3044, type='frame_decode', order=1), StyleMeta(name=' ロリ', id=3045, type='frame_decode', order=2)], speaker_uuid='0693554c-338e-4790-8982-b9c6d476dc69', version='0.16.1', order=16), CharacterMeta(name='小夜/SAYO', styles=[StyleMeta(name='ノーマル', id=3046, type='frame_decode', order=0)], speaker_uuid='a8cc6d22-aad0-4ab8-bf1e-2f843924164a', version='0.16.1', order=17), CharacterMeta(name='ナースロボ＿タイプＴ', styles=[StyleMeta(name='ノーマル', id=3047, type='frame_decode', order=0), StyleMeta(name='楽々', id=3048, type='frame_decode', order=1), StyleMeta(name='恐怖', id=3049, type='frame_decode', order=2)], speaker_uuid='882a636f-3bac-431a-966d-c5e6bba9f949', version='0.16.1', order=18), CharacterMeta(name='†聖騎士 紅桜†', styles=[StyleMeta(name='ノーマル', id=3051, type='frame_decode', order=0)], speaker_uuid='471e39d2-fb11-4c8c-8d89-4b322d2498e0', version='0.16.1', order=19), CharacterMeta(name='雀松朱司', styles=[StyleMeta(name='ノーマル', id=3052, type='frame_decode', order=0)], speaker_uuid='0acebdee-a4a5-4e12-a695-e19609728e30', version='0.16.1', order=20), CharacterMeta(name='麒ヶ島宗麟', styles=[StyleMeta(name='ノー マル', id=3053, type='frame_decode', order=0)], speaker_uuid='7d1e7ba7-f957-40e5-a3fc-da49f769ab65', version='0.16.1', order=21), CharacterMeta(name='春歌ナナ', styles=[StyleMeta(name='ノーマル', id=3054, type='frame_decode', order=0)], speaker_uuid='ba5d2428-f7e0-4c20-ac41-9dd56e9178b4', version='0.16.1', order=22), CharacterMeta(name='猫使アル', styles=[StyleMeta(name='ノーマル', id=3055, type='frame_decode', order=0), StyleMeta(name='おちつき', id=3056, type='frame_decode', order=1), StyleMeta(name='うきうき', id=3057, type='frame_decode', order=2)], speaker_uuid='00a5c10c-d3bd-459f-83fd-43180b521a44', version='0.16.1', order=23), CharacterMeta(name='猫使ビィ', styles=[StyleMeta(name='ノーマル', id=3058, type='frame_decode', order=0), StyleMeta(name='おちつき', id=3059, type='frame_decode', order=1)], speaker_uuid='c20a2254-0349-4470-9fc8-e5c0f8cf3404', version='0.16.1', order=24), CharacterMeta(name='中国うさぎ', styles=[StyleMeta(name='ノーマル', id=3061, type='frame_decode', order=0), StyleMeta(name='おどろき', id=3062, type='frame_decode', order=1), StyleMeta(name='こ わがり', id=3063, type='frame_decode', order=2), StyleMeta(name='へろへろ', id=3064, type='frame_decode', order=3)], speaker_uuid='1f18ffc3-47ea-4ce0-9829-0576d03a7ec8', version='0.16.1', order=25), CharacterMeta(name='栗田まろん', styles=[StyleMeta(name='ノーマル', id=3067, type='frame_decode', order=0)], speaker_uuid='04dbd989-32d0-40b4-9e71-17c920f2a8a9', version='0.16.1', order=26), CharacterMeta(name='あいえるたん', styles=[StyleMeta(name='ノーマル', id=3068, type='frame_decode', order=0)], speaker_uuid='dda44ade-5f9c-4a3a-9d2c-2a976c7476d9', version='0.16.1', order=27), CharacterMeta(name='満別花丸', styles=[StyleMeta(name='ノーマル', id=3069, type='frame_decode', order=0), StyleMeta(name='元気', id=3070, type='frame_decode', order=1), StyleMeta(name='ささやき', id=3071, type='frame_decode', order=2), StyleMeta(name='ぶりっ子', id=3072, type='frame_decode', order=3), StyleMeta(name='ボーイ', id=3073, type='frame_decode', order=4)], speaker_uuid='287aa49f-e56b-4530-a469-855776c84a8d', version='0.16.1', order=28), CharacterMeta(name='琴詠ニア', styles=[StyleMeta(name='ノ ーマル', id=3074, type='frame_decode', order=0)], speaker_uuid='97a4af4b-086e-4efd-b125-7ae2da85e697', version='0.16.1', order=29)]
[INFO] __main__: Creating an AudioQuery from Score(notes=[Note(frame_length=15, lyric='', key=None, id=None), Note(frame_length=45, lyric='ド', key=60, id=None), Note(frame_length=45, lyric='レ', key=62, id=None), Note(frame_length=45, lyric='ミ', key=64, id=None), Note(frame_length=15, lyric='', key=None, id=None)])
[INFO] __main__: Synthesizing with FrameAudioQuery(f0=[…], volume=[…], phonemes=[…], volume_scale=1.0, output_sampling_rate=24000, output_stereo=False)
[INFO] __main__: Wrote `output.wav`
```
