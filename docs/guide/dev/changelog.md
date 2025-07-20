- Keep a Changelogの方針に従い、[リリースの削除](https://github.com/VOICEVOX/voicevox_core/issues/1067)を行う際においても、[バージョンを"YANKED"と](https://keepachangelog.com/en/1.1.0/#yanked)した上でチェンジログとしての記述はそのまま残す。
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
- 日付についてはタグの日付（タグのannotateはしていないのでcommitter date）ではなく、GitHub Releaseの作成日を用いる。
    -  例えば`0.15.0-preview.15`は、コミット日の2023-11-07ではなく2023-11-13とする。
- 日付の形式は`yyyy-mm-dd (+09:00)`とする。確信が持てないが、この形が多分最も合法。
- 言語特有の変更点については\[C,Java,Python\]のように明記する。
- 破壊的変更は"\[BREAKING\]"を付ける。
- `0.16.0`より前については、次のように考える。
    - VOICEVOX ONNX Runtime以前の「OSS版VOICEVOX CORE」については考えない。Java APIは、製品版ビルドを出し始めた0.15.0-preview.12以降から存在することにする。

[ハイラムの法則]: https://www.hyrumslaw.com/
