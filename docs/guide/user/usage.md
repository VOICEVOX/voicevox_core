# VOICEVOX コア ユーザーガイド

## VOICEVOX コアとは

VOICEVOX の音声合成のコア部分で、VOICEVOX 音声合成が可能です。

コアを利用する方法は２つあります。動的ライブラリを使う方法と、各言語向けのライブラリを使う方法です。初心者の方は後者がおすすめです。

ここではまず環境構築の方法を紹介し、Python ライブラリのインストール方法を紹介します。その後、実際に音声合成を行う方法を少し細かく紹介します。

<!-- TODO: ソースコードを切り替え可能にして、説明文は全言語で共通にする -->

## 環境構築

### 実行に必要なファイルのダウンロード

コアを動作させるには依存ライブラリである VOICEVOX ONNX Runtime や、音声合成のための音声モデル（VVM ファイル）が必要です。これらはコア用の Downloader を用いてダウンロードすることができます。

> [!NOTE]
> 音声モデル（VVM ファイル）には利用規約が存在します。詳しくはダウンロードしたファイル内の README に記載されています。

[最新のリリース](https://github.com/VOICEVOX/voicevox_core/releases/latest/)から、お使いの環境にあった Downloader （例えば Windows の x64 環境の場合は`download-windows-x64.exe`）をダウンロードし、ファイル名を`download`に変更します。macOS や Linux の場合は実行権限を付与します。

```sh
# 実行権限の付与
chmod +x download
```

以下のコマンドで Downloader を実行して依存ライブラリとモデルをダウンロードします。DirectML 版や CUDA 版を利用する場合は引数を追加します。

```sh
# CPU版を利用する場合
./download --exclude c-api # C APIを使う場合は`--exclude c-api`は無し

# DirectML版を利用する場合
./download --exclude c-api --devices directml

# CUDA版を利用する場合
./download --exclude c-api --devices cuda
```

`voicevox_core`ディレクトリにファイル一式がダウンロードされています。以降の説明ではこのディレクトリで作業を行います。

詳細な Downloader の使い方は [こちら](./downloader.md) で紹介しています。また、GPUの使用については[こちら](./gpu.md)で説明しています。

<details>
<summary> Downloader を使わない場合</summary>

<!--
#### Raspberry Pi (armhf)の場合

Raspberry Pi 用の ONNX Runtime は以下からダウンロードできます。

- <https://github.com/VOICEVOX/onnxruntime-builder/releases>

動作には、libgomp のインストールが必要です。
-->

1. \[C APIのみ\] まず [Releases](https://github.com/VOICEVOX/voicevox_core/releases/latest) からダウンロードしたC APIライブラリ（`c-api`）の zip を、適当なディレクトリ名で展開します。CUDA 版、DirectML 版はかならずその zip ファイルをダウンロードしてください。
2. 同じく Releases から音声モデルの zip をダウンロードしてください。
3. [Open JTalk から配布されている辞書ファイル](https://jaist.dl.sourceforge.net/project/open-jtalk/Dictionary/open_jtalk_dic-1.11/open_jtalk_dic_utf_8-1.11.tar.gz) をダウンロードしてC APIライブラリを展開したディレクトリに展開してください。
4. CUDA や DirectML を利用する場合は、 [追加ライブラリ](https://github.com/VOICEVOX/voicevox_additional_libraries/releases/latest) をダウンロードして、C APIライブラリを展開したディレクトリに展開してください。

</details>

### Python ライブラリのインストール

> [!NOTE]
> Downloader を実行すればコアのC APIライブラリもダウンロードされているので、C APIライブラリを用いない場合はこの章はスキップできます。

`pip install`で Python ライブラリをインストールします。使いたい OS・アーキテクチャ・デバイス・バージョンによって URL が変わるので、[最新のリリース](https://github.com/VOICEVOX/voicevox_core/releases/latest/)の`Python wheel`に合わせます。

```sh
pip install https://github.com/VOICEVOX/voicevox_core/releases/download/[バージョン]/voicevox_core-[バージョン]+[デバイス]-cp310-abi3-[OS・アーキテクチャ].whl
```

## テキスト音声合成

VOICEVOX コアでは`Synthesizer`に音声モデルを読み込むことでテキスト音声合成できます。まずサンプルコードを紹介し、その後で処理１つ１つを説明します。

### サンプルコード

これは Python で書かれたサンプルコードですが、大枠の流れはどの言語でも同じです。

```python
from pprint import pprint
from voicevox_core.blocking import Onnxruntime, OpenJtalk, Synthesizer, VoiceModelFile

# 1. Synthesizerの初期化
voicevox_onnxruntime_path = "onnxruntime/lib/" + Onnxruntime.LIB_VERSIONED_FILENAME
open_jtalk_dict_dir = "dict/open_jtalk_dic_utf_8-1.11"
synthesizer = Synthesizer(Onnxruntime.load_once(filename=voicevox_onnxruntime_path), OpenJtalk(open_jtalk_dict_dir))

# 2. 音声モデルの読み込み
with VoiceModelFile.open("models/vvms/0.vvm") as model:
    synthesizer.load_voice_model(model)

# 3. テキスト音声合成
text = "サンプル音声です"
style_id = 0
wav = synthesizer.tts(text, style_id)
with open("output.wav", "wb") as f:
    f.write(wav)
```

### 1. Synthesizer の初期化

AIエンジンの`Onnxruntime`のインスタンスと、辞書などを取り扱う`OpenJtalk`のインスタンスを引数に渡して`Synthesizer`を初期化します。`Synthesizer`は音声合成だけでなく、音声モデルを複数読み込んだり、イントネーションのみを生成することもできます。

### 2. 音声モデルの読み込み

VVM ファイルから`VoiceModelFile`インスタンスを作成し、`Synthesizer`に読み込ませます。その VVM ファイルにどの声が含まれているかは`VoiceModelFile`の`.metas`や[音声モデルと声の対応表](https://github.com/VOICEVOX/voicevox_vvm/blob/main/README.md#%E9%9F%B3%E5%A3%B0%E3%83%A2%E3%83%87%E3%83%ABvvm%E3%83%95%E3%82%A1%E3%82%A4%E3%83%AB%E3%81%A8%E5%A3%B0%E3%82%AD%E3%83%A3%E3%83%A9%E3%82%AF%E3%82%BF%E3%83%BC%E3%82%B9%E3%82%BF%E3%82%A4%E3%83%AB%E5%90%8D%E3%81%A8%E3%82%B9%E3%82%BF%E3%82%A4%E3%83%AB-id-%E3%81%AE%E5%AF%BE%E5%BF%9C%E8%A1%A8)で確認できます。

```python
pprint(model.metas)
```

```txt
[SpeakerMeta(name='四国めたん',
             styles=[StyleMeta(name='ノーマル', id=2),
                     StyleMeta(name='あまあま', id=0),
                     StyleMeta(name='ツンツン', id=6),
                     StyleMeta(name='セクシー', id=4)],
             speaker_uuid='7ffcb7ce-00ec-4bdc-82cd-45a8889e43ff',
             version='0.14.4'),
 SpeakerMeta(name='ずんだもん',
             ...
```

### 3. テキスト音声合成

読み込んだ音声モデル内の声でテキスト音声合成を行います。`Synthesizer`の`.tts`にテキストとスタイル ID を渡すと、音声波形のバイナリデータが返ります。

## イントネーションの調整

`Synthesizer`はイントネーションの生成と音声合成の処理を分けることもできます。

詳しくは: [テキスト音声合成の流れ](./tts-process.md)

### AudioQuery の生成

まずテキストから`AudioQuery`を生成します。`AudioQuery`には各音の高さや長さが含まれています。

```python
text = "サンプル音声です"
style_id = 0
audio_query = synthesizer.create_audio_query(text, style_id)
pprint(audio_query)
```

```txt
AudioQuery(accent_phrases=[AccentPhrase(moras=[Mora(text='サ',
                                                    vowel='a',
                                                    vowel_length=0.13019563,
                                                    pitch=5.6954613,
                                                    consonant='s',
                                                    consonant_length=0.10374545),
                                               Mora(text='ン',
                                                    vowel='N',
                                                    vowel_length=0.07740324,
                                                    pitch=5.828728,
                                                    consonant=None,
                                                    consonant_length=None),
                                               Mora(text='プ',
                                                    ...
```

### AudioQuery の調整

少し声を高くしてみます。`AudioQuery`の`.pitch_scale`で声の高さを調整できます。

```python
audio_query.pitch_scale += 0.1
```

### 音声合成

調整した`AudioQuery`を`Synthesizer`の`.synthesis`に渡すと、調整した音声波形のバイナリデータが返ります。

```python
wav = synthesizer.synthesis(audio_query, style_id)
with open("output.wav", "wb") as f:
    f.write(wav)
```

`AudioQuery`で調整できるパラメータは他にも速さ`.speed_scale`や音量`.volume_scale`、音ごとの高さ`.accent_phrases[].moras[].pitch`などがあります。詳細は[API ドキュメント](https://voicevox.github.io/voicevox_core/apis/python_api/autoapi/voicevox_core/index.html#voicevox_core.AudioQuery)で紹介しています。

## ユーザー辞書

ユーザー辞書を扱うこともできます。

空の`UserDict`をコンストラクトし、そこに単語として`UserDictWord`を追加します。ユーザー辞書内の単語の編集と削除も可能です。

```python
from voicevox_core import UserDictWord
from voicevox_core.blocking import UserDict

user_dict = UserDict()
user_dict.add_word(UserDictWord("手札", "テフダ", 1, priority=6))
synthesizer.open_jtalk.use_user_dict(user_dict)

# 音声合成
text = "手札をシュサツと読んでしまうのをテフダにします"
style_id = 0
wav = synthesizer.tts(text, style_id)
with open("output.wav", "wb") as f:
    f.write(wav)
```

### ユーザー辞書の保存

```python
# ユーザー辞書を保存
user_dict.save("./user_dict.json")
```

```python
# 保存したユーザー辞書をロード
user_dict = UserDict()
user_dict.load("./user_dict.json")
```

## 非同期処理

非同期処理を行いたい場合は`voicevox_core.blocking`モジュールの代わりに`voicevox_core.asyncio`（Rustの場合は`voicevox_core::nonblocking`）モジュールを用います。

```python
from pprint import pprint
from voicevox_core.asyncio import Onnxruntime, OpenJtalk, Synthesizer, VoiceModelFile

# 1. Synthesizerの初期化
open_jtalk_dict_dir = "dict/open_jtalk_dic_utf_8-1.11"
synthesizer = Synthesizer(await Onnxruntime.load_once(), await OpenJtalk.new(open_jtalk_dict_dir))

# 2. 音声モデルの読み込み
async with await VoiceModelFile.open("models/vvms/0.vvm") as model:
    await synthesizer.load_voice_model(model)

# 3. テキスト音声合成
text = "サンプル音声です"
style_id = 0
wav = await synthesizer.tts(text, style_id)
with open("output.wav", "wb") as f:
    f.write(wav)
```

同じ音声モデルのインスタンスで同時に音声合成はできません（Mutex になっています）。

内部で利用する ONNX Runtime が最適化処理を行っているため、パフォーマンス目的で非同期処理するのは効果がないことが多いです。
`Synthesizer`の`cpu_num_threads`を減らした状態であれば、長い音声を合成しているものにロックされずバランシングできるかもしれません。

## その他のドキュメント

### APIドキュメント

[API ドキュメント](https://voicevox.github.io/voicevox_core/apis/)をご覧ください。

### サンプル実行

現在このリポジトリでは次のサンプルが提供されています。実行方法についてはそれぞれのディレクトリ内にある README を参照してください

- [Python(pip)](../../../example/python)
- [C++(UNIX CMake)](../../../example/cpp/unix)
- [C++(Windows Visual Studio)](../../../example/cpp/windows)

### 歌唱音声合成

歌唱音声については[歌唱音声合成](./song.md)をご覧ください。
