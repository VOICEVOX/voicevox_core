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
usage: run.py [-h] [--mode MODE] [--dict-dir DICT_DIR] [--text TEXT] [--out OUT] [--speaker-id SPEEKER_ID]

optional arguments:
  -h, --help            show this help message and exit
  --mode MODE           モード ("AUTO", "CPU", "GPU")
  --dict-dir DICT_DIR   Open JTalkの辞書ディレクトリ
  --text TEXT           読み上げさせたい文章
  --out OUT             出力wavファイルのパス
  --speaker-id SPEEKER_ID
                        話者IDを指定
```

## 実行例

```console
❯ cd voicevox_core/example/python
❯ python ./run.py
[DEBUG] run.py: voicevox_core.METAS=[Meta(name='四国めたん', styles=[Style(name='ノーマル', id=2), Style(name='あまあま', id=0), Style(name='ツンツン', id=6), Style(name='セクシー', id=4), Style(name='ささやき', id=36), Style(name='ヒソヒソ', id=37)], speaker_uuid='7ffcb7ce-00ec-4bdc-82cd-45a8889e43ff', version='0.14.3'), Meta(name='ずんだもん', styles=[Style(name='ノーマル', id=3), Style(name='あまあま', id=1), Style(name='ツンツン', id=7), Style(name='セクシー', id=5), Style(name='ささやき', id=22), Style(name='ヒソヒソ', id=38)], speaker_uuid='388f246b-8c41-4ac1-8e2d-5d79f3ff56d9', version='0.14.3'), Meta(name='春日部つむぎ', styles=[Style(name='ノーマル', id=8)], speaker_uuid='35b2c544-660e-401e-b503-0e14c635303a', version='0.14.3'), Meta(name='雨晴はう', styles=[Style(name='ノーマル', id=10)], speaker_uuid='3474ee95-c274-47f9-aa1a-8322163d96f1', version='0.14.3'), Meta(name='波音リツ', styles=[Style(name='ノーマル', id=9)], speaker_uuid='b1a81618-b27b-40d2-b0ea-27a9ad408c4b', version='0.14.3'), Meta(name='玄野武宏', styles=[Style(name='ノーマル', id=11), Style(name='喜び', id=39), Style(name='ツンギレ', id=40), Style(name='悲しみ', id=41)], speaker_uuid='c30dc15a-0992-4f8d-8bb8-ad3b314e6a6f', version='0.14.3'), Meta(name=' 白上虎太郎', styles=[Style(name='ふつう', id=12), Style(name='わーい', id=32), Style(name='びくびく', id=33), Style(name='おこ', id=34), Style(name='びえー ん', id=35)], speaker_uuid='e5020595-5c5d-4e87-b849-270a518d0dcf', version='0.14.3'), Meta(name='青山龍星', styles=[Style(name='ノーマル', id=13)], speaker_uuid='4f51116a-d9ee-4516-925d-21f183e2afad', version='0.14.3'), Meta(name='冥鳴ひまり', styles=[Style(name='ノーマル', id=14)], speaker_uuid='8eaad775-3119-417e-8cf4-2a10bfd592c8', version='0.14.3'), Meta(name='九州そら', styles=[Style(name='ノーマル', id=16), Style(name='あまあま', id=15), Style(name='ツンツン', id=18), Style(name='セクシー', id=17), Style(name='ささやき', id=19)], speaker_uuid='481fb609-6446-4870-9f46-90c4dd623403', version='0.14.3'), Meta(name='もち子さん', styles=[Style(name='ノーマル', id=20)], speaker_uuid='9f3ee141-26ad-437e-97bd-d22298d02ad2', version='0.14.3'), Meta(name='剣崎雌雄', styles=[Style(name='ノーマル', id=21)], speaker_uuid='1a17ca16-7ee5-4ea5-b191-2f02ace24d21', version='0.14.3'), Meta(name='WhiteCUL', styles=[Style(name='ノーマル', id=23), Style(name='たのしい', id=24), Style(name='かなしい', id=25), Style(name='びえーん', id=26)], speaker_uuid='67d5d8da-acd7-4207-bb10-b5542d3a663b', version='0.14.3'), Meta(name='後鬼', styles=[Style(name='人間ver.', id=27), Style(name='ぬいぐるみver.', id=28)], speaker_uuid='0f56c2f2-644c-49c9-8989-94e11f7129d0', version='0.14.3'), Meta(name='No.7', styles=[Style(name='ノーマル', id=29), Style(name='アナウンス', id=30), Style(name='読み聞かせ', id=31)], speaker_uuid='044830d2-f23b-44d6-ac0d-b5d733caa900', version='0.14.3'), Meta(name='ちび式じい', styles=[Style(name='ノーマル', id=42)], speaker_uuid='468b8e94-9da4-4f7a-8715-a22a48844f9e', version='0.14.3'), Meta(name='櫻歌ミコ', styles=[Style(name='ノーマル', id=43), Style(name='第二形態', id=44), Style(name='ロリ', id=45)], speaker_uuid='0693554c-338e-4790-8982-b9c6d476dc69', version='0.14.3'), Meta(name='小夜/SAYO', styles=[Style(name='ノーマル', id=46)], speaker_uuid='a8cc6d22-aad0-4ab8-bf1e-2f843924164a', version='0.14.3'), Meta(name='ナースロボ＿タイプＴ', styles=[Style(name='ノーマル', id=47), Style(name='楽々', id=48), Style(name='恐怖', id=49), Style(name='内緒話', id=50)], speaker_uuid='882a636f-3bac-431a-966d-c5e6bba9f949', version='0.14.3'), Meta(name='†聖騎士 紅桜†', styles=[Style(name='ノーマル', id=51)], speaker_uuid='471e39d2-fb11-4c8c-8d89-4b322d2498e0', version='0.14.3'), Meta(name='雀松朱司', styles=[Style(name='ノーマル', id=52)], speaker_uuid='0acebdee-a4a5-4e12-a695-e19609728e30', version='0.14.3'), Meta(name='麒ヶ島宗麟', styles=[Style(name='ノーマル', id=53)], speaker_uuid='7d1e7ba7-f957-40e5-a3fc-da49f769ab65', version='0.14.3'), Meta(name='春歌ナナ', styles=[Style(name='ノーマル', id=54)], speaker_uuid='ba5d2428-f7e0-4c20-ac41-9dd56e9178b4', version='0.14.3'), Meta(name='猫使アル', styles=[Style(name='ノーマル', id=55), Style(name='おちつき', id=56), Style(name='うきうき', id=57)], speaker_uuid='00a5c10c-d3bd-459f-83fd-43180b521a44', version='0.14.3'), Meta(name='猫使ビィ', styles=[Style(name='ノーマル', id=58), Style(name='おちつき', id=59), Style(name='人見知り', id=60)], speaker_uuid='c20a2254-0349-4470-9fc8-e5c0f8cf3404', version='0.14.3')]
[DEBUG] run.py: voicevox_core.SUPPORTED_DEVICES=SupportedDevices(cpu=True, cuda=False, dml=False)
[INFO] run.py: Initializing (acceleration_mode=<AccelerationMode.AUTO: 'AUTO'>, open_jtalk_dict_dir=PosixPath('voicevox_core/open_jtalk_dic_utf_8-1.11'))
[DEBUG] run.py: core.is_gpu_mode=False
[INFO] run.py: Loading model 0
[DEBUG] run.py: core.is_model_loaded(0)=True
[INFO] run.py: Creating an AudioQuery from 'この音声は、ボイスボックスを使用して、出力されています。'
[INFO] run.py: Synthesizing with {"accent_phrases": [{"moras": [{"text": "コ", "consonant": "k", "consonant_length": 0.07850838, "vowel": "o", "vowel_length": 0.060881548, "pitch": 5.485674}, {"text": "ノ", "consonant": "n", "consonant_length": 0.05698543, "vowel": "o", "vowel_length": 0.096929096, "pitch": 5.633757}], "accent": 2, "pause_mora": null, "is_interrogative": false}, {"moras": [{"text": "オ", "consonant": null, "consonant_length": null, "vowel": "o", "vowel_length": 0.12184215, "pitch": 5.8332877}, {"text": "ン", "consonant": null, "consonant_length": null, "vowel": "N", "vowel_length": 0.07362788, "pitch": 5.8688555}, {"text": "セ", "consonant": "s", "consonant_length": 0.07512727, "vowel": "e", "vowel_length": 0.079007246, "pitch": 5.723918}, {"text": "エ", "consonant": null, "consonant_length": null, "vowel": "e", "vowel_length": 0.071942694, "pitch": 5.596015}, {"text": "ワ", "consonant": "w", "consonant_length": 0.06436361, "vowel": "a", "vowel_length": 0.15232985, "pitch": 5.4623356}], "accent": 1, "pause_mora": {"text": "、", "consonant": null, "consonant_length": null, "vowel": "pau", "vowel_length": 0.3085951, "pitch": 0.0}, "is_interrogative": false}, {"moras": [{"text": "ボ", "consonant": "b", "consonant_length": 0.056357853, "vowel": "o", "vowel_length": 0.103436954, "pitch": 5.5773916}, {"text": "イ", "consonant": null, "consonant_length": null, "vowel": "i", "vowel_length": 0.06483335, "pitch": 5.7643595}, {"text": "ス", "consonant": "s", "consonant_length": 0.06817152, "vowel": "u", "vowel_length": 0.06812488, "pitch": 5.878236}, {"text": "ボ", "consonant": "b", "consonant_length": 0.049351893, "vowel": "o", "vowel_length": 0.10104511, "pitch": 5.8876576}, {"text": "ッ", "consonant": null, "consonant_length": null, "vowel": "cl", "vowel_length": 0.06349718, "pitch": 0.0}, {"text": "ク", "consonant": "k", "consonant_length": 0.0527189, "vowel": "U", "vowel_length": 0.055740334, "pitch": 0.0}, {"text": "ス", "consonant": "s", "consonant_length": 0.08895182, "vowel": "u", "vowel_length": 0.058778323, "pitch": 5.6777925}, {"text": "オ", "consonant": null, "consonant_length": null, "vowel": "o", "vowel_length": 0.10338681, "pitch": 5.514599}], "accent": 4, "pause_mora": null, "is_interrogative": false}, {"moras": [{"text": "シ", "consonant": "sh", "consonant_length": 0.064573064, "vowel": "i", "vowel_length": 0.090709515, "pitch": 5.4414697}, {"text": "ヨ", "consonant": "y", "consonant_length": 0.060504176, "vowel": "o", "vowel_length": 0.07323781, "pitch": 5.5361524}, {"text": "オ", "consonant": null, "consonant_length": null, "vowel": "o", "vowel_length": 0.08485783, "pitch": 5.6284075}], "accent": 3, "pause_mora": null, "is_interrogative": false}, {"moras": [{"text": "シ", "consonant": "sh", "consonant_length": 0.034160726, "vowel": "I", "vowel_length": 0.061993457, "pitch": 0.0}, {"text": "テ", "consonant": "t", "consonant_length": 0.071078844, "vowel": "e", "vowel_length": 0.13735397, "pitch": 5.667881}], "accent": 2, "pause_mora": {"text": "、", "consonant": null, "consonant_length": null, "vowel": "pau", "vowel_length": 0.33965635, "pitch": 0.0}, "is_interrogative": false}, {"moras": [{"text": "シュ", "consonant": "sh", "consonant_length": 0.058649763, "vowel": "U", "vowel_length": 0.063781895, "pitch": 0.0}, {"text": "ツ", "consonant": "ts", "consonant_length": 0.09303636, "vowel": "u", "vowel_length": 0.06302382, "pitch": 5.857424}, {"text": "リョ", "consonant": "ry", "consonant_length": 0.046520084, "vowel": "o", "vowel_length": 0.07469649, "pitch": 5.8819184}, {"text": "ク", "consonant": "k", "consonant_length": 0.052582815, "vowel": "U", "vowel_length": 0.04418713, "pitch": 0.0}], "accent": 2, "pause_mora": null, "is_interrogative": false}, {"moras": [{"text": "サ", "consonant": "s", "consonant_length": 0.07928567, "vowel": "a", "vowel_length": 0.07227267, "pitch": 5.5369396}, {"text": "レ", "consonant": "r", "consonant_length": 0.040197723, "vowel": "e", "vowel_length": 0.082754314, "pitch": 5.575339}, {"text": "テ", "consonant": "t", "consonant_length": 0.057140626, "vowel": "e", "vowel_length": 0.09039307, "pitch": 5.700317}], "accent": 3, "pause_mora": null, "is_interrogative": false}, {"moras": [{"text": "イ", "consonant": null, "consonant_length": null, "vowel": "i", "vowel_length": 0.064962484, "pitch": 5.674076}, {"text": "マ", "consonant": "m", "consonant_length": 0.071327865, "vowel": "a", "vowel_length": 0.09092417, "pitch": 5.6674485}, {"text": "ス", "consonant": "s", "consonant_length": 0.07241123, "vowel": "U", "vowel_length": 0.107922144, "pitch": 0.0}], "accent": 2, "pause_mora": null, "is_interrogative": false}], "speed_scale": 1.0, "pitch_scale": 0.0, "intonation_scale": 1.0, "volume_scale": 1.0, "pre_phoneme_length": 0.1, "post_phoneme_length": 0.1, "output_sampling_rate": 24000, "output_stereo": false, "kana": "コノ'/オ'ンセエワ、ボイスボ'ッ_クスオ/シヨオ'/_シテ'、_シュツ'リョ_ク/サレテ'/イマ'_ス"}
[INFO] run.py: Wrote `output.wav`
[DEBUG] lib.rs: Destructing a VoicevoxCore
```

正常に実行されれば音声合成の結果である wav ファイルが生成されます。
この例の場合、`"この音声は、ボイスボックスを使用して、出力されています。"`という読み上げの wav ファイルが output.wav という名前で生成されます。
