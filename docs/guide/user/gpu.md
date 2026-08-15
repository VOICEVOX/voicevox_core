# GPU の使用について

## CUDA

nvidia 製 GPU を搭載した Windows, Linux PC では CUDA を用いた合成が可能です。

CUDA 版を利用するには Downloader の実行が必要です。  
詳細は [CUDA 版をダウンロードする場合](./docs/guide/user/downloader.md#cuda) を参照してください

## DirectML

DirectX12 に対応した GPU を搭載した Windows PC では DirectML を用いた合成が可能です  
DirectML 版を利用するには Downloader の実行が必要です。  
詳細は [DirectML 版をダウンロードする場合](./docs/guide/user/downloader.md#directml) を参照してください

macOS の場合、CUDA の macOS サポートは現在終了しているため、VOICEVOX CORE の macOS 向けビルド済みライブラリも CUDA, CUDNN を利用しない CPU 版のみの提供となります。

## WebGPU

VOICEVOX CORE は ONNX Runtime の [WebGPU Execution Provider] にも対応しています。ただし現時点では、Downloader が取得する ONNX Runtime のビルド（[onnxruntime-builder]）に WebGPU Execution Provider は含まれていません。

WebGPU を用いた合成を行うには、WebGPU Execution Provider を有効にしてビルドした ONNX Runtime（`onnxruntime_USE_WEBGPU=ON`）を自前で用意し、[`Onnxruntime::load_once`]（もしくは `voicevox_onnxruntime_load_once`）にそのパスを渡してください。CUDA・DirectML と同様、対応する Execution Provider が利用できない場合は自動的に CPU にフォールバックします。

[WebGPU Execution Provider]: https://onnxruntime.ai/docs/execution-providers/WebGPU-ExecutionProvider.html
[onnxruntime-builder]: https://github.com/VOICEVOX/onnxruntime-builder
[`Onnxruntime::load_once`]: https://voicevox.github.io/voicevox_core/apis/rust_api/voicevox_core/blocking/struct.Onnxruntime.html#method.load_once

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
