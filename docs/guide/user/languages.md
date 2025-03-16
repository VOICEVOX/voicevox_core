## 各言語バインディングの機能

各言語バインディングには一部

| 言語 | パッケージレジストリ | 非同期API | [シリアライゼーション] | [ストリーミングAPI] |
| :- | :- | :- | :- | :-|
| Rust | :x: | :heavy_check_mark: | :heavy_check_mark: | :x: ([#972]) |
| C | :x: | :x: ([#1047]) | :heavy_check_mark: ([#975]) | :x: |
| Python | :x: ([#653], [#489]) | :heavy_check_mark: | :x: | :x: ([#972]) |
| Java | :x: ([#651]) | :x: ([#769]) | :heavy_check_mark: | :x: |

[シリアライゼーション]: ./serialization.md
[ストリーミングAPI]: https://github.com/VOICEVOX/voicevox_core/issues/853
[#972]: https://github.com/VOICEVOX/voicevox_core/pull/972
[#1047]: https://github.com/VOICEVOX/voicevox_core/issues/1047
[#975]: https://github.com/VOICEVOX/voicevox_core/issues/975
[#653]: https://github.com/VOICEVOX/voicevox_core/issues/653
[#489]: https://github.com/VOICEVOX/voicevox_core/issues/489
[#651]: https://github.com/VOICEVOX/voicevox_core/issues/651
[#769]: https://github.com/VOICEVOX/voicevox_core/issues/769
