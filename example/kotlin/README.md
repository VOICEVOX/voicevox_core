# Kotlin サンプルコード（jni-rs によるバインディング経由）

voicevox_core ライブラリ の Java バインディングを使った音声合成のサンプルコードです。

## 準備

1. VOICEVOX CORE Java API をビルドします。

crates/voicevox_core_java_api/README.md を参照してください。

2. ダウンローダーを使って環境構築します。

linux/mac の場合

download-linux-x64 のところはアーキテクチャや OS によって適宜読み替えてください。
https://github.com/VOICEVOX/voicevox_core/releases/latest#%E3%83%80%E3%82%A6%E3%83%B3%E3%83%AD%E3%83%BC%E3%83%80

```console
❯ binary=download-linux-x64
❯ curl -sSfL https://github.com/VOICEVOX/voicevox_core/releases/latest/download/${binary} -o download
❯ chmod +x download
❯ ./download -o ./example/kotlin --exclude c-api
```

windows の場合

```console
❯ Invoke-WebRequest https://github.com/VOICEVOX/voicevox_core/releases/latest/download/download-windows-x64.exe -OutFile ./download.exe
❯ ./download -o ./example/kotlin --exclude c-api
```

## 実行

Open JTalk 辞書ディレクトリ、読み上げさせたい文章、出力 wav ファイルのパスをオプションで指定することができます。

```console
❯ ./gradlew run --args="-h"
# または
❯ ./gradlew build
❯ java -jar ./app/build/libs/app-all.jar -h

Usage: voicevoxcoreexample options_list
Options:
    --mode [AUTO] -> モード { Value should be one of [auto, cpu, gpu] }
    --vvm -> vvmファイルへのパス (always required) { String }
    --onnxruntime [libonnxruntime.so.1.17.3] -> ONNX Runtimeのファイル名（モジュール名）もしくはファイルパス { String }
    --dictDir [./open_jtalk_dic_utf_8-1.11] -> Open JTalkの辞書ディレクトリ { String }
    --text [この音声は、ボイスボックスを使用して、出力されています。] -> 読み上げさせたい文章 { String }
    --out [./output.wav] -> 出力wavファイルのパス { String }
    --styleId [0] -> 話者IDを指定 { Int }
    --help, -h -> Usage info
```

## 実行例

<!-- FIXME: `cargo build -p test_util`するように案内するか、製品版のVVMを実行させる -->
<!-- FIXME: libvoicevox_onnxruntimeになったら`--onnxruntime`を指定するのではなく、`$LD_LIBRARY_PATH`とかに入れて実行するように案内する -->

```console
❯ # TODO: ダウンロード物の構成が変わったため色々壊れているはず
❯ # Linuxの場合
❯ ./gradlew run --args="--vvm ../../crates/test_util/data/model/sample.vvm --onnxruntime ../../crates/test_util/data/lib/libonnxruntime.so.1.17.3"
Inititalizing: AUTO, ../../crates/test_util/data/lib/libonnxruntime.so.1.17.3, ./open_jtalk_dic_utf_8-1.11
Loading: ../../crates/test_util/data/model/sample.vvm
Creating an AudioQuery from the text: この音声は、ボイスボックスを使用して、出力されています。
Synthesizing...
Saving the audio to ./output.wav
```
