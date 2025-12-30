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

## 実行

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

## 実行例

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
