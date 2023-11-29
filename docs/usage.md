# VOICEVOX CORE ユーザーガイド

## VOICEVOX CORE とは

VOICEVOX の音声合成のコア部分で、VOICEVOX 音声合成が可能です。

コアを利用する方法は２つあります。動的ライブラリを直接実行する方法と、各言語向けのライブラリをインストールする方法です。初心者の方は後者がおすすめです。

ここではまず環境構築の方法を紹介し、Python ライブラリのインストール方法を紹介します。その後、実際に音声合成を行う方法を少し細かく紹介します。

## 環境構築

### 実行に必要なファイルのダウンロード

コアを動作させるには依存ライブラリである `onnxruntime` や、音声合成のためのモデル(VVM)が必要です。これらはコア用の Downloader を用いてダウンロードすることができます。

[最新のリリース](https://github.com/VOICEVOX/voicevox_core/releases/latest/)から、お使いの環境にあった Downloader （Windows の x64 環境の場合は`download-windows-x64.exe`）をダウンロードし、ファイル名を`download`に変更します。macOS や Linux の場合は実行権限を付与します。

```sh
# 実行権限の付与
chmod +x download
```

以下のコマンドで Downloader を実行して依存ライブラリとモデルをダウンロードします。DirectML や CUDA を利用する場合は引数を追加します。

```sh
# CPU 版を利用する場合
./download

# DirectML を利用する場合
./download --device directml

# CUDA を利用する場合
./download --device cuda
```

`voicevox_core`ディレクトリにファイル一式がダウンロードされています。以降の説明ではこのディレクトリで作業を行います。

詳細な Downloader の使い方は [こちら](./downloader.md) で紹介しています。

### Python ライブラリのインストール

> [!NOTE]
> Downloader を実行すればコアの動的ライブラリもダウンロードされているので、Python ライブラリを用いない場合はこの章はスキップできます。

`pip install`で Python ライブラリをインストールします。使いたい OS・アーキテクチャ・デバイス・バージョンによって URL が変わるので、[最新のリリース](https://github.com/VOICEVOX/voicevox_core/releases/latest/)の`Python wheel`に合わせます。

```sh
pip install https://github.com/VOICEVOX/voicevox_core/releases/download/[バージョン]/voicevox_core-[バージョン]+[デバイス]-cp38-abi3-[OS・アーキテクチャ].whl
```

## 実行

Downloader を実行すればコアの動的ライブラリもダウンロードされているので、

環境構築はどうすればいいのか（Python で説明予定）
Synthesizer と VVM とは何なのか
実際に音声合成する方法
調整を変更する方法
キャラクターを変える方法
