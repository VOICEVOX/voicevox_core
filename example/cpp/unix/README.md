# C++ サンプルコード（Linux・macOS 向け）

voicevox_core ライブラリを C++ から使用するサンプルコード (`talk.cpp`) です。ビルドするために C++ の開発環境（CMake 3.16 以上を含む）が必要です。

## 必要なファイルの準備

まず、この README があるディレクトリで、[Downloader を使用して voicevox_core をダウンロードします](../../../docs/guide/user/downloader.md#default)。

## ビルド

以下のコマンドを実行すると、`build` ディレクトリが作成され、ビルド産物がその中に生成されます：

```bash
cmake -S . -B build
cmake --build build
```

## `talk`（テキスト合成音声）の実行

`build` ディレクトリ以下にできた実行ファイル (`talk`) をこのディレクトリにコピーしてから実行します：

```bash
cp build/talk ./
# ./talk <読み上げさせたい文章>
./talk これはテストです
```

正常に実行されれば `audio.wav` が生成されます。以下のコマンドですぐに聞くことができます：

```bash
# Linux の場合
aplay audio.wav

# macOS の場合
afplay audio.wav
```

## `song`（歌唱合成音声）の実行

`talk`と同様の形で実行します。ただし`song`は引数を取りません。

```bash
cp build/song ./
# talkと違い、songは引数を取らない
./song
# audio.wavが生成されます。
```
