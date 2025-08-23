# [CHANGELOG.md]の書き方の方針

この方針は今のところ人間のために、というよりメンテナである[@qryxip]の忘備録として書いている。LLMにやらせることは今のところ計画されていない。

- [オリジナルのKeep a Changelog]を拡大解釈する形で、次のルールを定める。
    - 日付の形式は`yyyy-mm-dd (+09:00)`とする。確信が持てないが、この形が多分最も合法。
    - 日付についてはタグの日付（タグのannotateはしていないのでcommitter date）を用いる。ただし、`0.16.0-preview.0`より前についてはGitHub Releasesの日付を使う（後述）。
        - `0.16.0-preview.0`以降であれば、余程のことが無い限りこの二つは同じ日になるはず。
    - 日本語で記述し、'.'の代わりに'。'を用いる。
    - 変更には必ず、該当する一つ以上のPRを記載する。リリース時点で[@VOICEVOX]に所属していないメンバーからのPRは、`#1111 by @contributor`のように表記する。co-authorとして含まれる場合も"helped by"として名前を入れる。
        - 「該当する」の基準が微妙だが、とりあえずケースバイケースで緩く考える。
        - Copilotや[@claude]の場合どうするのかは、その時になったら考える。
    - 現時点でまだAPIをパブリックにしていない機能、例えばストリーミングAPIやソング機能などは、HTMLコメントでのメモを残しておく。
- Keep a Changelogの方針に従い、[リリースの削除]を行う際においても、[バージョンを"YANKED"と]した上でチェンジログとしての記述はそのまま残す。
    > Yanked releases are versions that (中略). Often these versions don't even appear in change logs. They should.
- 以下のものの変更を、Keep a Changelogが言うところの"notable"なものとする（[ハイラムの法則]を胸に、できる限り広く捉えるようにする）。
    - パブリックAPI
    - **ユーザー用**ドキュメント
    - exampleコード
    - CD (≠ CI)の内容物
    - パフォーマンス改善やバグフィックスなどの、外から見える挙動
- 以下のものの変更は"notable"ではないものとする。
    - **内部**ドキュメント
    - リファクタ
    - テスト
    - CI (≠ CD)
- compatible\_engine関係の変更は無視する。
- 古いバージョンの変更履歴については、必要に応じて、未来の視点からのコメントを「補足:」という形で入れておく。
- 言語特有の変更点については"\[C,Python,Java\]"のように明記する。順番は"Rust"、"C"、"Python"、"Java"、"ダウンローダー"。
- OS特有の変更点についても"\[macOS,Linux\]"のように明記する。言語とは別に書く。順番は"Windows"、"macOS"、"Linux"、"Android"、"iOS"。
- 破壊的変更は"\[BREAKING\]"を付ける。
- 場合によって :tada: を付ける。
- `0.16.1`以降については、次のように考える。
    - `<p>`や`<details>`の利用を避け、噛み砕いた説明は[../user/key-changes/](../user/key-changes/)に任せる。
- `0.16.1`より前については、次のように考える。
    - `<p>`や、場合によっては`<details>`も積極的に用いる。
- `0.16.0-preview.0`より前については、次のように考える。
    - VOICEVOX ONNX Runtime以前の「OSS版VOICEVOX CORE」については考えない。Java APIは、製品版ビルドを出し始めた0.15.0-preview.12以降から存在することにする。
    - 日付についてはタグの日付ではなく、GitHub Releaseの作成日を用いる。
        -  例えば`0.15.0-preview.15`は、コミット日の2023-11-07ではなく2023-11-13とする。
- `0.15.0-preview.16`より前については、次のように考える。
    - 変更量が膨大なため、ベストエフォートを心掛ける。また、できる限り簡潔に書くようにする。

[CHANGELOG.md]: ../../../CHANGELOG.md
[@qryxip]: https://github.com/qryxip
[オリジナルのKeep a Changelog]: https://keepachangelog.com/en/1.1.0/
[@VOICEVOX]: https://github.com/VOICEVOX
[@claude]: https://github.com/claude
[リリースの削除]: https://github.com/VOICEVOX/voicevox_core/issues/1067
[バージョンを"YANKED"と]: https://keepachangelog.com/en/1.1.0/#yanked
[ハイラムの法則]: https://www.hyrumslaw.com/
