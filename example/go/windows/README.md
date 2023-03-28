# Go サンプルコード（Windows 向け）

voicevox_core ライブラリを Go から使用するサンプルコード (`main.go`) です。ビルドするために Go の開発環境が必要です。

## 必要なファイルの準備

voicevox_core のダウンローダーを任意のディレクトリで実行してください。(ダウンローダーの説明:https://github.com/VOICEVOX/voicevox_core/blob/main/docs/downloads/download.md)
生成されたフォルダ(voicevox_core)内のファイル・フォルダのうち、以下を本ディレクトリに配置してください。

```
model
open_jtalk_dic_utf_8-1.11
onnxruntime.dll
voicevox_core.dll
```

## ビルド,実行

以下のコマンドを実行すると、`simple_tts.exe`が作成、実行されます:

```shell
go build
./simple_tts
```

正常に実行されれば `speech.wav` が生成されます。以下のコマンドですぐに聞くことができます：

```shell
$player = New-Object Media.SoundPlayer "./speech.wav"
$player.Play()
```
