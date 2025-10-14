# 貢献者ガイド

> [!NOTE]
> まだ策定中です。

Issue を解決するプルリクエストを作成される際は、別の方と同じ Issue に取り組むことを避けるため、
Issue 側で取り組み始めたことを伝えるか、最初に Draft プルリクエストを作成してください。

[VOICEVOX 非公式 Discord サーバー](https://discord.gg/WMwWetrzuh)にて、開発の議論や雑談を行っています。気軽にご参加ください。

## Rust 以外の言語の API に関する方針

[APIデザイン ガイドライン](./docs/guide/dev/api-design.md)をご覧ください。

[cbindgen](https://crates.io/crates/cbindgen) が手元にインストールされているなら、それを使いヘッダファイルを生成することもできます。

## CHANGELOG.mdの更新

プルリクエストにて[CHANGELOG.md](./CHANGELOG.md)も同時に更新するかどうかは任意です。しない場合は、メンテナが後から行います (例: [#1172](https://github.com/VOICEVOX/voicevox_core/pull/1172))。

## テスト

テストの方法は各言語ごとに異なります。各言語のreadmeを参照してください。

Rustのコードに対しては一般的なRustライブラリと同様、`cargo test`でテストできます。

```bash
cargo test # Rust APIのテストを実行
```

[`--include-ignored`]を付けることで[C API]のテストも一緒に実行できます。

```bash
cargo test -- --include-ignored # Rust APIとC APIをまとめてテスト
```

[`--include-ignored`]: https://doc.rust-lang.org/reference/attributes/testing.html#the-ignore-attribute
[C API]: ./crates/voicevox_core_c_api/

## ダウンローダーの実行

```bash
cargo run -p downloader

# ヘルプを表示
cargo run -p downloader -- -h
```

## タイポチェック

[typos](https://github.com/crate-ci/typos) を使ってタイポのチェックを行っています。
[typos をインストール](https://github.com/crate-ci/typos#install) した後

```bash
typos
```

## プルリクエストにおけるCI

[Benchmarksワークフロー]の実行の都合上、たまに`CodSpeed Performance Analysis`が`failure`になることがあります。そうなった場合はアクセス権を持つ誰かが、Benchmarksワークフローをre-runします。実際にパフォーマンスが低下していると考えられる場合は、プルリクエスト内での議論ののち権限のある誰かが[「リグレッションの承認」]を行います。

[Benchmarksワークフロー]: ./.github/workflows/benchmarks.yml
[「リグレッションの承認」]: https://codspeed.io/docs/features/performance-checks#acknowledge-regressions-or-benchmark-drops
