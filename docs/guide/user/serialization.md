## データのシリアライゼーション

現在、[`AudioQuery`]と[`UserDict`]をシリアライズするときのスキーマはVOICEVOX ENGINEと同じになっています。

- Rust APIではSerde、Java APIではGSONでの変換時にスキーマに従います。
- C APIではRust APIのSerde実装に従ったJSONの入出力を行っています。
- :construction: Python APIではシリアライズ関係のAPIについては議論中の段階であり、まだ実装されていません。`dataclasses.asdict`などを用いてシリアライズを試みることは避けてください。

ただし今後の設計については議論中であり、今後の破壊的変更にて変更される可能性があります（[#1049 (comment)]）。

[`AudioQuery`]: https://voicevox.github.io/voicevox_core/apis/python_api/autoapi/voicevox_core/index.html#voicevox_core.AudioQuery
[`UserDict`]: https://voicevox.github.io/voicevox_core/apis/python_api/autoapi/voicevox_core/blocking/index.html#voicevox_core.blocking.UserDict
[#1049 (comment)]: https://github.com/VOICEVOX/voicevox_core/pull/1049#issuecomment-2763230417
