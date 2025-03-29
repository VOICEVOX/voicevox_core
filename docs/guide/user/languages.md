## 各言語バインディングの機能

<!-- TODO: 自然言語という意味と間違えられそうなので、bindings.mdとかにする？ -->

各言語バインディングには一部実装されていない機能があります。以下がその表です。

| 言語 | パッケージレジストリ | 非同期API | [シリアライゼーション] | [ストリーミングAPI] |
| :- | :- | :- | :- | :-|
| Rust | :x: | :heavy_check_mark: | :warning: ¹ | :x: ([#972]) |
| C | :x: | :x: ([#1047]) | :warning: ¹ ([#975]) | :x: |
| Python | :x: ([#653], [#489]) | :heavy_check_mark: | :x: | :x: ([#972]) |
| Java | :x: ([#651]) | :x: ([#769]) | :warning: ¹ | :x: |

¹ 設計について議論中であり、今後の破壊的変更にて変更される可能性があります。

[シリアライゼーション]: ./serialization.md
[ストリーミングAPI]: https://github.com/VOICEVOX/voicevox_core/issues/853
[#972]: https://github.com/VOICEVOX/voicevox_core/pull/972
[#1047]: https://github.com/VOICEVOX/voicevox_core/issues/1047
[#975]: https://github.com/VOICEVOX/voicevox_core/issues/975
[#653]: https://github.com/VOICEVOX/voicevox_core/issues/653
[#489]: https://github.com/VOICEVOX/voicevox_core/issues/489
[#651]: https://github.com/VOICEVOX/voicevox_core/issues/651
[#769]: https://github.com/VOICEVOX/voicevox_core/issues/769
