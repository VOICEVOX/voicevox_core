# リリース手順

1. [`build`ワークフロー]を`コード署名する`付きで実行し、draft releaseを作成。
2. 一つ前のリリースを参考にしてのbodyを書く。「{バージョン名}の主要な変更点」については[key-changes]から写す。
3. draftを解除し、latestに。
4. [CHANGELOG.md]にバージョンを刻む。

1.において:

- `tag_name`を設定するところが失敗する場合、人間が手で設定することでリリース作業を続行してもよい（may）。
- `download_test`が失敗する場合、リリース成果物やダウンローダーの機能に問題がないと考えるのなら失敗を許容してリリース作業を続行してもよい（may）。

[`build`ワークフロー]: https://github.com/VOICEVOX/voicevox_core/actions/workflows/build.yml
[key-changes]: ../user/key-changes
[CHANGELOG.md]: ../../../CHANGELOG.md
