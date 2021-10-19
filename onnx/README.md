# ONNX BUILD

1. https://github.com/microsoft/onnxruntime/releases から環境に対応したonnxruntimeをダウンロードし展開

	例：Windows(x64アーキテクチャ)でCPU版を作るならonnxruntime-win-x64-1.9.0.zip

2. `mkdir build`でビルド用ディレクトリを作成し、`build`ディレクトリに移動
3. `cmake .. -DONNXRUNTIME_DIR="../onnxruntime-win-x64-gpu-1.9.0"`を実行。`-DONNXRUNTIME_DIR`引数は1.で展開したフォルダへのパスを指定する。

	ここでGPU版を作りたいときは引数`-DENABLE_CUDA=ON`を追加する

4. `cmake --build . --config Release`でビルド
5. `cmake --install .`とすると`./lib`フォルダにdll/libが生成される

## Note
core.dllはonnxruntime.dllに依存しているため、読み込むときはonnxruntime.dllがシステムから見えていなければならない

```
> dumpbin /dependents core.dll
Microsoft (R) COFF/PE Dumper Version 14.29.30038.1
Copyright (C) Microsoft Corporation.  All rights reserved.


Dump of file core.dll

File Type: DLL

  Image has the following dependencies:

    onnxruntime.dll
    MSVCP140.dll
    VCRUNTIME140.dll
    VCRUNTIME140_1.dll
    api-ms-win-crt-runtime-l1-1-0.dll
    api-ms-win-crt-heap-l1-1-0.dll
    api-ms-win-crt-string-l1-1-0.dll
    api-ms-win-crt-locale-l1-1-0.dll
    KERNEL32.dll

...(省略)
```
