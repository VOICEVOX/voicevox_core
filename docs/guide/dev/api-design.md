# APIデザイン ガイドライン

## ENGINE API との関係

VOICEVOX CORE の API は ENGINE API をベースにしています。ただし CORE ユーザーにとって使いやすくなる相応の理由があれば、ENGINE からの乖離を許容します。

ENGINE 側に存在しない機能が CORE にあることは問題ありません。

## Rust 以外の言語の API

VOICEVOX CORE の主要機能は Rust で実装されることを前提としており、他の言語のラッパーでのみの機能追加はしない方針としています。これは機能の一貫性を保つための方針です。

ただし機能追加ではない範囲で、各言語の習慣に適合するような変更は積極的に行っていきます。例えば:

* [`StyleId`](https://voicevox.github.io/voicevox_core/apis/rust_api/voicevox_core/struct.StyleId.html)といった[newtype](https://rust-unofficial.github.io/patterns/patterns/behavioural/newtype.html)は、そのままnewtypeとして表現するべきです。
    * 例えばPythonなら[`typing.NewType`](https://docs.python.org/ja/3/library/typing.html#newtype)で表現します。
* オプショナルな引数は、キーワード引数がある言語であればキーワード引数で、ビルダースタイルが一般的な言語であればビルダースタイルで表現すべきです。
* 列挙型は、PythonやTypeScriptでは文字列リテラルの合併型で表現するべきです。
* [`VoiceModelFile`](https://voicevox.github.io/voicevox_core/apis/rust_api/voicevox_core/nonblocking/struct.VoiceModelFile.html)の"close"後でも`id`と`metas`は利用可能であるべきです。ただしRustにおける"close"だけは、`VoiceModelFile`を`id`と`metas`に分解するような形にします。
* `Synthesizer::render`は`range: std::ops::Range<usize>`を引数に取っています。`Range<usize>`にあたる型が標準で存在し、かつそれが配列の範囲指定として用いられるようなものであれば、それを使うべきです。
    * ただし例えばPythonでは、`slice`を引数に取るのは慣習にそぐわないため`start: int, stop: int`のようにすべきです。
    * もし`Range<usize>`にあたる型が標準で無く、かつRustの"start"/"end"やPythonの"start"/"stop"にあたる明確な言葉が無いのであれば、誤解が生じないよう"end_exclusive"のように命名するべきです。
