# VOICEVOX CORE

[VOICEVOX](https://voicevox.hiroshiba.jp/) の音声合成コア。  
[Releases](https://github.com/VOICEVOX/voicevox_core/releases) にビルド済みのコアライブラリ（.so/.dll/.dylib）があります。

（エディターは [VOICEVOX](https://github.com/VOICEVOX/voicevox/) 、
エンジンは [VOICEVOX ENGINE](https://github.com/VOICEVOX/voicevox_engine/) 、
全体構成は [こちら](https://github.com/VOICEVOX/voicevox/blob/main/docs/%E5%85%A8%E4%BD%93%E6%A7%8B%E6%88%90.md) に詳細があります。）

## 環境構築

downloader を用いて環境構築を行う場合

### Windows の場合

powershell で下記コマンドを実行してください

```powershell
Invoke-WebRequest https://github.com/VOICEVOX/voicevox_core/releases/latest/download/Download.ps1 | powershell
```

### Linux,MacOS の場合

```bash
curl -sSLo https://github.com/VOICEVOX/voicevox_core/releases/latest/download/download.sh | bash -s
```

詳細な downloader の使い方については [こちら](./docs/downloads/Download.md) を参照してください

<details>
<summary> downloader を使わない場合</summary>

#### GPU を使用する場合

<!--
#### Raspberry Pi (armhf)の場合

Raspberry Pi 用の ONNX Runtime は以下からダウンロードできます。

- <https://github.com/VOICEVOX/onnxruntime-builder/releases>

動作には、libgomp のインストールが必要です。
-->

### コアライブラリのダウンロードと配置

1. まず [Releases](https://github.com/VOICEVOX/voicevox_core/releases/latest) からダウンロードしたコアライブラリの zip を、適当なディレクトリ名で展開します。CUDA 版、DirectML 版はかならずその zip ファイルをダウンロードしてください。
2. CUDA,DirectML を利用するのであれば [追加ライブラリ](https://github.com/VOICEVOX/voicevox_additional_libraries/releases/latest) をダウンロードして展開したディレクトリに展開する。
3. tts 機能を使うのであれば次に [open_jtalk 公式](https://jaist.dl.sourceforge.net/project/open-jtalk/Dictionary/open_jtalk_dic-1.11/open_jtalk_dic_utf_8-1.11.tar.gz) から open_jtalk の辞書ファイルをダウンロードしてコアライブラリを展開したディレクトリに展開する。

</details>

### 注意

#### GPU の使用について

##### CUDA

nvidia 製 GPU を搭載した Windows, Linux PC では CUDA を用いた合成が可能です。

CUDA 版を利用するには専用の download コマンドの実行が必要です。  
詳細は [CUDA 版をダウンロードする場合](./docs/downloads/Download.md#cuda-版をダウンロードする場合) を参照してください

##### DirectML

DirectX12 に対応した GPU を搭載した Windows PC では DirectML を用いた合成が可能です  
DirectML 版を利用するには専用の download コマンドの実行が必要です。  
詳細は [DirectML 版をダウンロードする場合](./docs/downloads/Download.md#directml-版をダウンロードする場合) を参照してください

MacOS の場合、CUDA の macOS サポートは現在終了しているため、VOICEVOX CORE の macOS 向けコアライブラリも CUDA, CUDNN を利用しない CPU 版のみの提供となります。

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

まず Open JTalk 辞書フォルダを配置します。 http://open-jtalk.sourceforge.net/ を開き、Dictionary for Open JTalk 欄の Binary Package (UTF-8)をクリックして「open_jtalk_dic_utf_8-1.11.tar.gz」をダウンロードします。
これを展開してできた「open_jtalk_dic_utf_8-1.11」フォルダを example/python に配置します。

- バージョン 0.12 以降の voicevox_core, onnxruntime ライブラリ（配布ページ: https://github.com/VOICEVOX/voicevox_core/releases ）を example/python に配置する
  - Linux の場合：`voicevox_core-linux-{お使いのCPUアーキテクチャ}-cpu-{バージョン}.zip` 内の 全ての so file
  - macOS の場合：`voicevox_core-osx-{お使いのCPUアーキテクチャ}-cpu-{バージョン}.zip` 内の 全ての dylib file
  - Windows の場合：`voicevox_core-windows-{お使いのCPUアーキテクチャ}-cpu-{バージョン}.zip` 内の 全ての dll file

```bash
cd example/python

# サンプルコード実行のための依存モジュールのインストール
pip install -r requirements.txt
python run.py \
    --text "これは本当に実行できているんですか" \
    --speaker_id 1

# 引数の紹介
# --text 読み上げるテキスト
# --speaker_id 話者ID
# --use_gpu GPUを使う
# --f0_speaker_id 音高の話者ID（デフォルト値はspeaker_id）
# --f0_correct 音高の補正値（デフォルト値は0。+-0.3くらいで結果が大きく変わります）
```

### その他の言語

サンプルコードを実装された際はぜひお知らせください。こちらに追記させて頂きます。

## API

[Releases](https://github.com/VOICEVOX/voicevox_core/releases)にある zip ファイル内に core.h が入っているのでご確認ください

## コアライブラリのビルド

[Releases](https://github.com/VOICEVOX/voicevox_core/releases) にあるビルド済みのコアライブラリを利用せず、自分で一からビルドする場合こちらを参照してください。ビルドには [Rust](https://www.rust-lang.org/ja) ([Windows での Rust 開発環境構築手順はこちら](https://docs.microsoft.com/ja-jp/windows/dev-environment/rust/setup)) と [cmake](https://cmake.org/download/) が必要です。

model フォルダにある onnx モデルはダミーのため、ノイズの混じった音声が出力されます

```bash
cargo build --release
```

## コアライブラリのテスト

```bash
cargo test
```

## 事例紹介

**[VOICEVOX ENGINE SHARP](https://github.com/yamachu/VoicevoxEngineSharp) [@yamachu](https://github.com/yamachu)** ･･･ VOICEVOX ENGINE の C# 実装  
**[Node VOICEVOX Engine](https://github.com/y-chan/node-voicevox-engine) [@y-chan](https://github.com/y-chan)** ･･･ VOICEVOX ENGINE の Node.js/C++ 実装

## ライセンス

ソースコードのライセンスは [MIT LICENSE](./LICENSE) です。

[Releases](https://github.com/VOICEVOX/voicevox_core/releases) にあるビルド済みのコアライブラリは別ライセンスなのでご注意ください。
