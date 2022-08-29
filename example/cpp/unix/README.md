# C++ サンプルコード（Linux・macOS 向け）

voicevox_core ライブラリを C++ から使用するサンプルコード (`simple_tts.cpp`) です。ビルドするために C++ の開発環境（CMake 3.16 以上を含む）が必要です。

## サンプル実行方法

まず、この README があるディレクトリで、[Downloader を使用して voicevox_core をダウンロードします](../../../docs/downloads/download.md#default)。

## ビルド

以下のコマンドを実行すると、`build` ディレクトリが作成され、ビルド産物がその中に生成されます：

```bash
cmake -S . -B build
cmake --build build
```

## 実行

`build` ディレクトリ以下にできた実行ファイル (`simple_tts`) をこのディレクトリにコピーしてから実行します：

```bash
cp build/simple_tts ./
# ./simple_tts <読み上げさせたい文章>
./simple_tts これはテストです
```

正常に実行されれば `audio.wav` が生成されます。以下のコマンドですぐに聞くことができます：

```bash
# Linux の場合
aplay audio.wav

# macOS の場合
afplay audio.wav
```
