# voicevox_core_node_api

VOICEVOX CORE の Node.js バインディングです。

## 環境構築

以下の環境が必要です。

- Rustup
- Node > 10.0.0

## ファイル構成

```console
.
├── build.rs
├── Cargo.toml           : Rust プロジェクトとしてのマニフェストファイルです。
├── exports              : blocking および promises 名前空間下のクラスを再エクスポートするためのモジュールです。
│   ├── blocking.d.ts
│   ├── blocking.js
│   ├── promises.d.ts
│   └── promises.js
├── package.json
├── README.md
├── src                  : Rust で書かれたソースファイルです。
│   └── ...
├── __test__             : TypeScript と AVA によるテストのソースファイルです。
│   └── ...
├── tsconfig.json
└── yarn.lock
```

## ビルド

npm で依存ライブラリをダウンロードし、`build` タスクによって `index.d.ts`, `index.js` および `node` 拡張子のネイティブモジュールを生成します。

```console
❯ npm install
❯ npm run build
```

## テスト

`npm test` でテストを行うことができます。

```console
❯ npm test
```

## ドキュメント

`npm run docs` でドキュメントを生成できます。

```console
❯ npm run docs
```
