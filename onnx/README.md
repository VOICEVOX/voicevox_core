# How to install
## 1. ONNX BUILD

1. https://github.com/microsoft/onnxruntime/releases から環境に対応したonnxruntimeをダウンロードし展開

	例：Windows(x64アーキテクチャ)でCPU版を作るならonnxruntime-win-x64-1.9.0.zip

2. `mkdir build`でビルド用ディレクトリを作成し、`build`ディレクトリに移動
3. `cmake .. -DONNXRUNTIME_DIR="../onnxruntime-win-x64-gpu-1.9.0"`を実行。`-DONNXRUNTIME_DIR`引数は1.で展開したフォルダへのパスを指定する。

	ここでGPU版を作りたいときは引数`-DENABLE_CUDA=ON`を追加する

4. `cmake --build . --config Release`でビルド
5. `cmake --install .`とすると`./lib`フォルダにdll/libが生成される

## 2. setuptools

1. ONNX BUILDで生成した`lib`フォルダの中身を`python`ディレクトリに入れる
2. `python`ディレクトリ内で`python setup.py install`を実行
