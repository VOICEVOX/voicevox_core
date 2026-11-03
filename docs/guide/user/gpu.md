# GPU の使用について

VOICEVOX CORE は、ONNX Runtime の [Execution Provider](https://onnxruntime.ai/docs/execution-providers/)（EP）を通して GPU を利用します。各 EP を動かすのに必要な環境（GPU ドライバや、CUDA・cuDNN・DirectML のライブラリなど）については、ONNX Runtime のドキュメントを参照してください。

[Downloader](./downloader.md) の`--devices`を使うと、その EP に対応した VOICEVOX ONNX Runtime と、EP が必要とするライブラリ（[voicevox_additional_libraries](https://github.com/VOICEVOX/voicevox_additional_libraries)）をまとめてダウンロードできます。これは環境を揃えるための補助的な手段であり、必要なライブラリを自前で用意しても構いません。

## CUDA

NVIDIA 製 GPU を搭載した Windows, Linux PC では CUDA を用いた合成が可能です。

ONNX Runtime の [CUDA Execution Provider](https://onnxruntime.ai/docs/execution-providers/CUDA-ExecutionProvider.html) に対応します。必要な環境についてはそちらを参照してください。

Downloader を使う場合は [CUDA 版をダウンロードする場合](./downloader.md#cuda) を参照してください。

## DirectML

DirectX 12 に対応した GPU を搭載した Windows PC では DirectML を用いた合成が可能です。

ONNX Runtime の [DirectML Execution Provider](https://onnxruntime.ai/docs/execution-providers/DirectML-ExecutionProvider.html) に対応します。必要な環境についてはそちらを参照してください。

Downloader を使う場合は [DirectML 版をダウンロードする場合](./downloader.md#directml) を参照してください。

## macOS

CUDA の macOS サポートは現在終了しているため、VOICEVOX CORE の macOS 向けビルド済みライブラリも CUDA, cuDNN を利用しない CPU 版のみの提供となります。

<!--
## Raspberry Piでの使用について

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
