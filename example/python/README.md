# Python サンプルコード (PyO3 によるバインディング経由)

voicevox_core ライブラリ の Python バインディングを使った音声合成のサンプルコードです。  
`pip install`で導入することができます。

## 準備

1. wheelパッケージのインストールをします。

`[バージョン]`の部分は適宜書き換えてください。

```console
❯ pip install https://github.com/VOICEVOX/voicevox_core/releases/download/[バージョン]/voicevox_core-[バージョン]+cpu-cp38-abi3-linux_x86_64.whl
```

cpu-cp38-abi3-linux_x86_64のところはアーキテクチャやOSによって適宜読み替えてください。
https://github.com/VOICEVOX/voicevox_core/releases/latest


2. ダウンローダーを使って環境構築します。

linux/macの場合

download-linux-x64のところはアーキテクチャやOSによって適宜読み替えてください。
https://github.com/VOICEVOX/voicevox_core/releases/latest#%E3%83%80%E3%82%A6%E3%83%B3%E3%83%AD%E3%83%BC%E3%83%80

```console
❯ binary=download-linux-x64
❯ curl -sSfL https://github.com/VOICEVOX/voicevox_core/releases/latest/download/${binary} -o download
❯ chmod +x download
❯ ./download -o ./example/python
❯ # いくつかのファイルは不要なので消すことができます
❯ #rm -r ./example/python/{model,VERSION,*voicevox_core*}
```

windowsの場合

```console
❯ Invoke-WebRequest https://github.com/VOICEVOX/voicevox_core/releases/latest/download/download-windows-x64.exe -OutFile ./download.exe
❯ ./download -o ./example/python
❯ # いくつかのファイルは不要なので消すことができます
❯ #Remove-Item -Recurse ./example/python/model,./example/python/VERSION,./example/python/*voicevox_core*
```

TODO:

- Python インタプリタ ≧3.8 + venv

## 実行

run.py を実行します。 Open JTalk 辞書ディレクトリ、読み上げさせたい文章、出力 wav ファイルのパスをオプションで指定することができます。

```console
❯ python ./run.py -h
usage: run.py [-h] [--mode MODE] [--dict-dir DICT_DIR] [--text TEXT] [--out OUT] [--speaker-id SPEAKER_ID] vvm

positional arguments:
  vvm                   vvmファイルへのパス

optional arguments:
  -h, --help            show this help message and exit
  --mode MODE           モード ("AUTO", "CPU", "GPU")
  --dict-dir DICT_DIR   Open JTalkの辞書ディレクトリ
  --text TEXT           読み上げさせたい文章
  --out OUT             出力wavファイルのパス
  --speaker-id SPEAKER_ID
                        話者IDを指定
```

## 実行例

```console
❯ python ./run.py ../../model/sample.vvm
[DEBUG] __main__: voicevox_core.create_supported_devices()=SupportedDevices(cpu=True, cuda=False, dml=False)
[INFO] __main__: Initializing (acceleration_mode=<AccelerationMode.AUTO: 'AUTO'>, open_jtalk_dict_dir=PosixPath('open_jtalk_dic_utf_8-1.11'))
[DEBUG] __main__: synthesizer.metas=[]
[DEBUG] __main__: synthesizer.is_gpu_mode=False
[INFO] __main__: Loading `../../model/sample.vvm`
[INFO] __main__: Creating an AudioQuery from 'この音声は、ボイスボックスを使用して、出力されています。'
[INFO] __main__: Synthesizing with {"accent_phrases": [{"moras": [{"text": "コ", "consonant": "k", "consonant_length": 0.0556899, "vowel": "o", "vowel_length": 0.075180575, "pitch": 5.542309}, {"text": "ノ", "consonant": "n", "consonant_length": 0.06551014, "vowel": "o", "vowel_length": 0.09984577, "pitch": 5.6173983}], "accent": 2, "pause_mora": null, "is_interrogative": false}, {"moras": [{"text": "オ", "consonant": null, "consonant_length": null, "vowel": "o", "vowel_length": 0.116150305, "pitch": 5.7063766}, {"text": "ン", "consonant": null, "consonant_length": null, "vowel": "N", "vowel_length": 0.044380233, "pitch": 5.785717}, {"text": "セ", "consonant": "s", "consonant_length": 0.07719758, "vowel": "e", "vowel_length": 0.08653869, "pitch": 5.662092}, {"text": "エ", "consonant": null, "consonant_length": null, "vowel": "e", "vowel_length": 0.08311573, "pitch": 5.532917}, {"text": "ワ", "consonant": "w", "consonant_length": 0.06373148, "vowel": "a", "vowel_length": 0.16219379, "pitch": 5.293258}], "accent": 1, "pause_mora": {"text": "、", "consonant": null, "consonant_length": null, "vowel": "pau", "vowel_length": 0.35826492, "pitch": 0.0}, "is_interrogative": false}, {"moras": [{"text": "ボ", "consonant": "b", "consonant_length": 0.047082342, "vowel": "o", "vowel_length": 0.12611786, "pitch": 5.583892}, {"text": "イ", "consonant": null, "consonant_length": null, "vowel": "i", "vowel_length": 0.059451744, "pitch": 5.7947493}, {"text": "ス", "consonant": "s", "consonant_length": 0.089278996, "vowel": "u", "vowel_length": 0.11847979, "pitch": 5.818695}, {"text": "ボ", "consonant": "b", "consonant_length": 0.06535433, "vowel": "o", "vowel_length": 0.120458946, "pitch": 5.7965107}, {"text": "ッ", "consonant": null, "consonant_length": null, "vowel": "cl", "vowel_length": 0.06940381, "pitch": 0.0}, {"text": "ク", "consonant": "k", "consonant_length": 0.053739145, "vowel": "U", "vowel_length": 0.05395376, "pitch": 0.0}, {"text": "ス", "consonant": "s", "consonant_length": 0.10222931, "vowel": "u", "vowel_length": 0.071811065, "pitch": 5.8024883}, {"text": "オ", "consonant": null, "consonant_length": null, "vowel": "o", "vowel_length": 0.11092262, "pitch": 5.5036163}], "accent": 4, "pause_mora": null, "is_interrogative": false}, {"moras": [{"text": "シ", "consonant": "sh", "consonant_length": 0.09327768, "vowel": "i", "vowel_length": 0.09126951, "pitch": 5.369444}, {"text": "ヨ", "consonant": "y", "consonant_length": 0.06251812, "vowel": "o", "vowel_length": 0.07805054, "pitch": 5.5021667}, {"text": "オ", "consonant": null, "consonant_length": null, "vowel": "o", "vowel_length": 0.09904325, "pitch": 5.5219536}], "accent": 3, "pause_mora": null, "is_interrogative": false}, {"moras": [{"text": "シ", "consonant": "sh", "consonant_length": 0.04879771, "vowel": "I", "vowel_length": 0.06514315, "pitch": 0.0}, {"text": "テ", "consonant": "t", "consonant_length": 0.0840496, "vowel": "e", "vowel_length": 0.19438823, "pitch": 5.4875555}], "accent": 2, "pause_mora": {"text": "、", "consonant": null, "consonant_length": null, "vowel": "pau", "vowel_length": 0.35208154, "pitch": 0.0}, "is_interrogative": false}, {"moras": [{"text": "シュ", "consonant": "sh", "consonant_length": 0.05436731, "vowel": "U", "vowel_length": 0.06044446, "pitch": 0.0}, {"text": "ツ", "consonant": "ts", "consonant_length": 0.102865085, "vowel": "u", "vowel_length": 0.057028636, "pitch": 5.6402535}, {"text": "リョ", "consonant": "ry", "consonant_length": 0.058293864, "vowel": "o", "vowel_length": 0.080050275, "pitch": 5.6997967}, {"text": "ク", "consonant": "k", "consonant_length": 0.054767884, "vowel": "U", "vowel_length": 0.042932786, "pitch": 0.0}], "accent": 2, "pause_mora": null, "is_interrogative": false}, {"moras": [{"text": "サ", "consonant": "s", "consonant_length": 0.08067487, "vowel": "a", "vowel_length": 0.07377973, "pitch": 5.652378}, {"text": "レ", "consonant": "r", "consonant_length": 0.040600352, "vowel": "e", "vowel_length": 0.079322875, "pitch": 5.6290326}, {"text": "テ", "consonant": "t", "consonant_length": 0.06773268, "vowel": "e", "vowel_length": 0.08347456, "pitch": 5.6427326}], "accent": 3, "pause_mora": null, "is_interrogative": false}, {"moras": [{"text": "イ", "consonant": null, "consonant_length": null, "vowel": "i", "vowel_length": 0.07542324, "pitch": 5.641289}, {"text": "マ", "consonant": "m", "consonant_length": 0.066299975, "vowel": "a", "vowel_length": 0.107257664, "pitch": 5.6201453}, {"text": "ス", "consonant": "s", "consonant_length": 0.07186453, "vowel": "U", "vowel_length": 0.1163103, "pitch": 0.0}], "accent": 2, "pause_mora": null, "is_interrogative": false}], "speed_scale": 1.0, "pitch_scale": 0.0, "intonation_scale": 1.0, "volume_scale": 1.0, "pre_phoneme_length": 0.1, "post_phoneme_length": 0.1, "output_sampling_rate": 24000, "output_stereo": false, "kana": "コノ'/オ'ンセエワ、ボイスボ'ッ_クスオ/シヨオ'/_シテ'、_シュツ'リョ_ク/サレテ'/イマ'_ス"}
[INFO] __main__: Wrote `output.wav`
[DEBUG] voicevox_core_python_api: Destructing a VoicevoxCore
```

正常に実行されれば音声合成の結果である wav ファイルが生成されます。
この例の場合、`"この音声は、ボイスボックスを使用して、出力されています。"`という読み上げの wav ファイルが output.wav という名前で生成されます。
