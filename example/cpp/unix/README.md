# C++ サンプルコード（Linux・macOS 向け）

voicevox_core ライブラリを C++ から使用するサンプルコード (`simple_tts.cpp`) です。ビルドするために C++ の開発環境（CMake 3.16 以上を含む）が必要です。

## 必要なファイルの準備

- Open JTalk の辞書（配布ページ: http://open-jtalk.sourceforge.net/ ）
  - 配布ページの "Dictionary for Open JTalk" 欄にある "Binary Package (UTF-8)" からダウンロードしてください
  - 解凍してできた `open_jtalk_dic_utf_8-1.11` フォルダをそのままこのディレクトリに配置してください

以上の準備を終えると、本ディレクトリには以下のファイル・フォルダが存在することになります：

```
CMakeLists.txt
open_jtalk_dic_utf_8-1.11
simple_tts.cpp
```

## ビルド

以下のコマンドを実行すると、`build` ディレクトリが作成され、ビルド産物がその中に生成されます：

```bash
cmake -S . -B build
cmake --build build
```

## 実行

`build` ディレクトリ以下にできた実行ファイル (`simple_tts`) をこのディレクトリにコピーしてから実行します：

```bash
LD_LIBRARY_PATH=./build:$LD_LIBRARY_PATH build/simple_tts これはテストです
```

正常に実行されれば `audio.wav` が生成されます。以下のコマンドですぐに聞くことができます：

```bash
# Linux の場合
aplay audio.wav

# macOS の場合
afplay audio.wav
```
