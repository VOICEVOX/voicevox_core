## データのシリアライゼーション

[`AudioQuery`]と[`UserDict`]をシリアライズするときのスキーマはVOICEVOX ENGINEと同じになっています。Rust APIではSerde、Java APIではGSONでの変換時にスキーマに従います。

- C APIではスキーマに従ったJSONの入出力を行っています。今後の設計については議論中です（[#975]）。
- :construction: Python APIではシリアライズ関係のAPIについては議論中の段階であり、まだ実装されていません。`dataclasses.asdict`などを用いてシリアライズを試みることは避けてください。

[`AudioQuery`]: https://voicevox.github.io/voicevox_core/apis/python_api/autoapi/voicevox_core/index.html#voicevox_core.AudioQuery
[`UserDict`]: https://voicevox.github.io/voicevox_core/apis/python_api/autoapi/voicevox_core/blocking/index.html#voicevox_core.blocking.UserDict
[#975]: https://github.com/VOICEVOX/voicevox_core/issues/975
