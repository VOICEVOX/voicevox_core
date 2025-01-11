# voicevox\_core\_python\_api

VOICEVOX CORE の Python バインディングです。

## 環境構築

以下の環境が必要です。

- Rustup
- Python ≧ 3.10
- Poetry ≧ 1.6

```console
❯ poetry install --with dev
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

## トラブルシューティング

Maturinで依存クレート（例: [open\_jtalk-sys](https://github.com/VOICEVOX/open_jtalk-rs)）のビルドが失敗する場合は、Maturinの外であらかじめ `cargo build` すれば解決する場合があります。リンクまでは上手くいかないかもしれませんが、該当の依存クレートまではビルドできるかもしれません。

```console
❯ cargo build -p voicevox_core_python_api [--releasee]
```
