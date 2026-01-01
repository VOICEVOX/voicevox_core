# リリース手順

1. [`build_and_deploy`ワークフロー]を`コード署名する`付きで実行し、prereleaseを作成。
2. 一つ前のリリースを参考にしてのbodyを書く。「{バージョン名}の主要な変更点」については[key-changes]から写す。
3. prereleaseを解除し、latestに。
4. [CHANGELOG.md]にバージョンを刻む。

[`build_and_deploy`ワークフロー]: https://github.com/qryxip/voicevox_core/actions/workflows/build_and_deploy.yml
[key-changes]: ../user/key-changes
[CHANGELOG.md]: ../../../CHANGELOG.md
