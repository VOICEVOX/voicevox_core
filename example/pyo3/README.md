# Python サンプルコード (PyO3 によるバインディング経由)

voicevox_core ライブラリ の Python バインディングを使った音声合成のサンプルコードです。  
`pip install`で導入することができます。

## 準備

TODO

- Python インタプリタ ≧3.8 + venv
- voicevox_core_python_api の whl (`pip install`)
- onnxruntime の DLL (/README.md と同様)
- open_jtalk_dic_utf_8-1.11 (/README.md と同様)

## 実行

Open JTalk 辞書ディレクトリ、読み上げさせたい文章、出力 wav ファイルのパスの 3 つを指定して run.py を実行します。

```console
❯ python ./run.py -h
usage: run.py [-h] [--mode MODE] open_jtalk_dict_dir text out

positional arguments:
  open_jtalk_dict_dir  Open JTalkの辞書ディレクトリ
  text                 読み上げさせたい文章
  out                  出力wavファイルのパス

optional arguments:
  -h, --help           show this help message and exit
  --mode MODE          モード ("AUTO", "CPU", "GPU")
```

```console
❯ # python ./run.py <Open JTalk辞書ディレクトリ> <読み上げさせたい文章> <出力wavファイルのパス>
❯ python ./run.py ./open_jtalk_dic_utf_8-1.11 これはテストです ./audio.wav
[DEBUG] run.py: voicevox_core.METAS=[Meta(name='四国めたん', styles=[Style(name='あまあま', id=0)], speaker_uuid='7ffcb7ce-00ec-4bdc-82cd-45a8889e43ff', version='0.0.1'), Meta(name='ずんだもん', styles=[Style(name='あまあま', id=1)], speaker_uuid='388f246b-8c41-4ac1-8e2d-5d79f3ff56d9', version='0.0.1')]
[DEBUG] run.py: voicevox_core.SUPPORTED_DEVICES=SupportedDevices(cpu=True, cuda=True, dml=False)
[INFO] run.py: Initializing (acceleration_mode=<AccelerationMode.AUTO: 'AUTO'>, open_jtalk_dict_dir=PosixPath('open_jtalk_dic_utf_8-1.11'))
[DEBUG] run.py: core.is_gpu_mode=True
[INFO] run.py: Loading model 0
[DEBUG] run.py: core.is_model_loaded(0)=True
[INFO] run.py: Creating an AudioQuery from 'これはテストです'
[INFO] run.py: Synthesizing with {"accent_phrases": [{"moras": [{"text": "コ", "consonant": "k", "consonant_length": 0.063058704, "vowel": "o", "vowel_length": 0.08937682, "pitch": 5.5699596}, {"text": "レ", "consonant": "r", "consonant_length": 0.047547057, "vowel": "e", "vowel_length": 0.07596417, "pitch": 5.6643105}, {"text": "ワ", "consonant": "w", "consonant_length": 0.053706698, "vowel": "a", "vowel_length": 0.10348523, "pitch": 5.7773285}], "accent": 3, "pause_mora": null, "is_interrogative": false}, {"moras": [{"text": "テ", "consonant": "t", "consonant_length": 0.06311223, "vowel": "e", "vowel_length": 0.07596652, "pitch": 5.881741}, {"text": "ス", "consonant": "s", "consonant_length": 0.038565055, "vowel": "U", "vowel_length": 0.050694168, "pitch": 0.0}, {"text": "ト", "consonant": "t", "consonant_length": 0.06685759, "vowel": "o", "vowel_length": 0.0753997, "pitch": 5.737323}, {"text": "デ", "consonant": "d", "consonant_length": 0.058399618, "vowel": "e", "vowel_length": 0.09201351, "pitch": 5.4747167}, {"text": "ス", "consonant": "s", "consonant_length": 0.08852549, "vowel": "U", "vowel_length": 0.1281984, "pitch": 0.0}], "accent": 1, "pause_mora": null, "is_interrogative": false}], "speed_scale": 1.0, "pitch_scale": 0.0, "intonation_scale": 1.0, "volume_scale": 1.0, "pre_phoneme_length": 0.1, "post_phoneme_length": 0.1, "output_sampling_rate": 24000, "output_stereo": false, "kana": "コレワ'/テ'_ストデ_ス"}
[INFO] run.py: Wrote `audio.wav`
[DEBUG] lib.rs: Destructing a VoicevoxCore
```

正常に実行されれば音声合成の結果である wav ファイルが生成されます。
この例の場合、`"これはテストです"`という読み上げの wav ファイルが audio.wav という名前で生成されます。
