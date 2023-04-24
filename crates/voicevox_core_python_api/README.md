# voicevox\_core\_python\_api

VOICEVOX CORE の Python バインディングです。

## 環境構築

以下の環境が必要です。

- Rustup
- Python ≧3.8の venv
    - `maturin>=0.13.2,<0.14`

[Maturin](https://maturin.rs/)は venv にインストールする必要があります。
適当な場所に venv を作って下さい。

```console
❯ python -m venv ../../.venv
```

```console
❯ ../../.venv/Scripts/Activate.ps1 (Windows)
```

```console
❯ ../../.venv/bin/activate (maxOS/Linux)
```

venv を作ったらその venv 上で Maturin をインストールします。

```console
# maturinのインストール
❯ pip install -r ./requirements.txt
```

## ファイル構成

```console
.
├── Cargo.toml                        : Rustプロジェクトとしてのマニフェストファイルです。
├── pyproject.toml
├── python                            : このディレクトリの内容がwhlに入ります。
│   └── voicevox_core
│       ├── __init__.py
│       ├── _load_dlls.py
│       ├── _models.py
│       ├── __pycache__               : maturin developで生成されます。
│       │   └── …
│       ├── py.typed
│       ├── _rust.abi3.{dll,dylib,so} : maturin developで生成されるpydファイルです。
│       └── _rust.pyi                 : _rust.abi3.{dll,dylib,so}用のpyiファイルです。
├── README.md
├── requirements.txt
└── src                               : Rustのソースコードです。_rust.abi3.{dll,dylib,so}にコンパイルされます。
    └── lib.rs
```

## ビルド

`maturin develop` で Rust のコードが pyd として python/voicevox\_core 下に保存された後、`editable` なパッケージとしてインストールされます。

```console
❯ maturin develop --locked
```

`maturin build` で whl としてビルドすることができます。

```console
❯ maturin build --release --locked
```

## サンプル実行

`maturin develop` で editable な状態でインストールした後、[example/python](../../example/python) にてサンプルを実行できます。
