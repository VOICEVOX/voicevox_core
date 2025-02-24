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
