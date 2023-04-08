# VOICEVOX CORE

[![dependency status](https://deps.rs/repo/github/VOICEVOX/voicevox_core/status.svg)](https://deps.rs/repo/github/VOICEVOX/voicevox_core)

[VOICEVOX](https://voicevox.hiroshiba.jp/) の音声合成コア。  
[Releases](https://github.com/VOICEVOX/voicevox_core/releases) にビルド済みのコアライブラリ（.so/.dll/.dylib）があります。

（エディターは [VOICEVOX](https://github.com/VOICEVOX/voicevox/) 、
エンジンは [VOICEVOX ENGINE](https://github.com/VOICEVOX/voicevox_engine/) 、
全体構成は [こちら](https://github.com/VOICEVOX/voicevox/blob/main/docs/%E5%85%A8%E4%BD%93%E6%A7%8B%E6%88%90.md) に詳細があります。）

## 環境構築

Downloader を用いて環境構築を行う場合

### Windows の場合

PowerShell で下記コマンドを実行してください

```PowerShell
Invoke-WebRequest https://github.com/VOICEVOX/voicevox_core/releases/latest/download/download-windows-x64.exe -OutFile ./download.exe
./download.exe
```

### Linux/macOS の場合

[最新のリリース](https://github.com/VOICEVOX/voicevox_core/releases/latest)から環境に合わせてダウンローダーのバイナリをダウンロードしてください。
現在利用可能なのは以下の 4 つです。

- download-linux-arm64
- download-linux-x64
- download-osx-arm64
- download-osx-x64

以下は Linux の x64 での実行例です。

```bash
binary=download-linux-x64
curl -sSfL https://github.com/VOICEVOX/voicevox_core/releases/latest/download/${binary} -o download
chmod +x download
./download
```

詳細な Downloader の使い方については [こちら](./docs/downloads/download.md) を参照してください

<details>
<summary> Downloader を使わない場合</summary>

<!--
#### Raspberry Pi (armhf)の場合

Raspberry Pi 用の ONNX Runtime は以下からダウンロードできます。

- <https://github.com/VOICEVOX/onnxruntime-builder/releases>

動作には、libgomp のインストールが必要です。
-->

1. まず [Releases](https://github.com/VOICEVOX/voicevox_core/releases/latest) からダウンロードしたコアライブラリの zip を、適当なディレクトリ名で展開します。CUDA 版、DirectML 版はかならずその zip ファイルをダウンロードしてください。
2. [Open JTalk から配布されている辞書ファイル](https://jaist.dl.sourceforge.net/project/open-jtalk/Dictionary/open_jtalk_dic-1.11/open_jtalk_dic_utf_8-1.11.tar.gz) をダウンロードしてコアライブラリを展開したディレクトリに展開してください。
3. CUDA や DirectML を利用する場合は、 [追加ライブラリ](https://github.com/VOICEVOX/voicevox_additional_libraries/releases/latest) をダウンロードして、コアライブラリを展開したディレクトリに展開してください。

</details>

### 注意

#### GPU の使用について

##### CUDA

nvidia 製 GPU を搭載した Windows, Linux PC では CUDA を用いた合成が可能です。

CUDA 版を利用するには Downloader の実行が必要です。  
詳細は [CUDA 版をダウンロードする場合](./docs/downloads/download.md#cuda) を参照してください

##### DirectML

DirectX12 に対応した GPU を搭載した Windows PC では DirectML を用いた合成が可能です  
DirectML 版を利用するには Downloader の実行が必要です。  
詳細は [DirectML 版をダウンロードする場合](./docs/downloads/download.md#directml) を参照してください

macOS の場合、CUDA の macOS サポートは現在終了しているため、VOICEVOX CORE の macOS 向けコアライブラリも CUDA, CUDNN を利用しない CPU 版のみの提供となります。

<!--
#### Raspberry Piでの使用について

Raspberry PiなどのarmhアーキテクチャPCでの使用では、環境構築時に https://github.com/VOICEVOX/onnxruntime-builder/releases にある独自ビルドのonnxruntimeを使用する必要があります。
そのため、環境にあったファイルのURLを取得し、上記例の代わりに
```bash
python configure.py --ort_download_link <独自ビルドonnxruntimeのURL>
```
を実行してください

また、動作には、libgomp のインストールが必要です。

```shell
sudo apt install libgomp1
```
-->

## サンプル実行

現在このリポジトリでは次のサンプルが提供されています。実行方法についてはそれぞれのディレクトリ内にある README を参照してください

- [Python(pip)](./example/python)
- [C++(UNIX CMake)](./example/cpp/unix)
- [C++(Windows Visual Studio)](./example/cpp/windows)

### その他の言語

- [Go(Windows)](https://github.com/yerrowTail/voicevox_core_go_sample) @yerrowTail

サンプルコードを実装された際はぜひお知らせください。こちらに追記させて頂きます。

## API

[API ドキュメント](https://voicevox.github.io/voicevox_core/apis/c_api/globals_func.html)をご覧ください。

## コアライブラリのビルド

[Releases](https://github.com/VOICEVOX/voicevox_core/releases) にあるビルド済みのコアライブラリを利用せず、自分で一からビルドする場合こちらを参照してください。ビルドには [Rust](https://www.rust-lang.org/ja) ([Windows での Rust 開発環境構築手順はこちら](https://docs.microsoft.com/ja-jp/windows/dev-environment/rust/setup)) と [cmake](https://cmake.org/download/) が必要です。

model フォルダにある onnx モデルはダミーのため、ノイズの混じった音声が出力されます

```bash
# DLLをビルド
cargo build --release -p voicevox_core_c_api
```

```bash
# DLL用のヘッダファイルvoicevox_core.hを生成
# cbindgenが手元にインストールされているのならそちらでも可
cargo xtask generate-c-header
```

## コアライブラリのテスト

```bash
cargo test
```

## タイポチェック

[typos](https://github.com/crate-ci/typos) を使ってタイポのチェックを行っています。
[typos をインストール](https://github.com/crate-ci/typos#install) した後

```bash
typos
```

## 事例紹介

**[voicevox.rb](https://github.com/sevenc-nanashi/voicevox.rb) [@sevenc-nanashi](https://github.com/sevenc-nanashi)** ･･･ VOICEVOX CORE の Ruby 向け FFI ラッパー  
**[Node VOICEVOX Engine](https://github.com/y-chan/node-voicevox-engine) [@y-chan](https://github.com/y-chan)** ･･･ VOICEVOX ENGINE の Node.js/C++ 実装  
**[VOICEVOX ENGINE SHARP](https://github.com/yamachu/VoicevoxEngineSharp) [@yamachu](https://github.com/yamachu)** ･･･ VOICEVOX ENGINE の C# 実装  
**[voicevoxcore4s](https://github.com/windymelt/voicevoxcore4s) [@windymelt](https://github.com/windymelt)** ･･･ VOICEVOX CORE の Scala(JVM) 向け FFI ラッパー  
**[voicevox_flutter](https://github.com/char5742/voicevox_flutter) [@char5742](https://github.com/char5742)** ･･･ VOICEVOX CORE の Flutter 向け FFI ラッパー  
## ライセンス

ソースコードのライセンスは [MIT LICENSE](./LICENSE) です。

[Releases](https://github.com/VOICEVOX/voicevox_core/releases) にあるビルド済みのコアライブラリは別ライセンスなのでご注意ください。
