# ONNXバックエンドコア
ONNX RuntimeをバックエンドにしたVOICEVOX CoreのPythonライブラリを作成します。

## 0a. モデルの準備
https://github.com/Hiroshiba/vv_core_inference でonnx版に変換したモデルを用意します。
このディレクトリにある`model/`フォルダの中には、[release-0.0.1](https://github.com/Hiroshiba/vv_core_inference/releases/tag/0.0.1)のonnx版が既に用意されています。

## 0b. 必須pythonライブラリのインストール
このリポジトリのexample/python内にある[requirements.txt](../example/python/requirements.txt)にあるライブラリをインストールする。

```
cd ../example/python
pip install -r requirements.txt
```

## 1. コアバイナリの作成
このREADMEがあるonnxディレクトリで作業する。

```
# https://github.com/microsoft/onnxruntime/releases から環境に対応したonnxruntimeをダウンロードし展開する。
# 例：Windows(x64アーキテクチャ)でCPU版を作るならonnxruntime-win-x64-1.9.0.zip
wget https://github.com/microsoft/onnxruntime/releases/download/v1.9.0/onnxruntime-win-x64-1.9.0.zip
unzip onnxruntime-win-x64-1.9.0.zip

# ビルド用ディレクトリを作成し移動
mkdir build
cd build

# cmake configurationを実行。`-DONNXRUNTIME_DIR`引数は先ほど展開したonnxruntimeへのパスを指定する
# ここでGPU版を作りたいときは引数`-DENABLE_CUDA=ON`を追加する
cmake .. -DONNXRUNTIME_DIR="../onnxruntime-win-x64-1.9.0"

# ビルド・インストール。libフォルダに必要なライブラリ等が生成される
cmake --build . --config Release
cmake --install .
```

## 2. Pythonライブラリの作成

1. 上記で生成した`lib`フォルダの**中身**を全て`./python/core`ディレクトリに入れる
2. `python`ディレクトリ内で`pip install .`を実行

## 3. エンジンの起動
このリポジトリの`example/python/run.py`を実行するとき、カレントディレクトリにonnxモデルがないと動かないことに注意する。

このREADMEがあるonnxディレクトリで作業する。
```
cd model

python ../../example/python/run.py \
    --text "これは本当に実行できているんですか" \
    --speaker_id 1
```

この`run.py`のオプションの詳細は[トップのREADME](../README.md)を参照。