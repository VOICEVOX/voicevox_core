# テキスト音声合成の流れ

テキスト音声合成の流れを図にするとこのようになります。

```mermaid
flowchart TD;
    ja-txt[日本語のテキスト]
    ap-without-mora-data[音高・音素長<b>抜きの</b><br><a href="https://voicevox.github.io/voicevox_core/apis/python_api/autoapi/voicevox_core/index.html#voicevox_core.AccentPhrase">アクセント句</a>の列]
    ap-with-mora-data[音高・音素長<b>入りの</b><br><a href="https://voicevox.github.io/voicevox_core/apis/python_api/autoapi/voicevox_core/index.html#voicevox_core.AccentPhrase">アクセント句</a>の列]
    aq[<a href="https://voicevox.github.io/voicevox_core/apis/python_api/autoapi/voicevox_core/index.html#voicevox_core.AudioQuery">AudioQuery</a>]
    wav[WAV形式の音声]

    ja-txt -->|<a href="https://voicevox.github.io/voicevox_core/apis/python_api/autoapi/voicevox_core/blocking/index.html#voicevox_core.blocking.OpenJtalk.analyze">OpenJtalk.analyze</a>| ap-without-mora-data
           -->|<ul><li><a href="https://voicevox.github.io/voicevox_core/apis/python_api/autoapi/voicevox_core/blocking/index.html#voicevox_core.blocking.Synthesizer.replace_phoneme_length">Synthesizer.replace_phoneme_length</a></li><li><a href="https://voicevox.github.io/voicevox_core/apis/python_api/autoapi/voicevox_core/blocking/index.html#voicevox_core.blocking.Synthesizer.replace_mora_pitch">Synthesizer.replace_mora_pitch</a></li><ul>| ap-with-mora-data
           -->|<a href="https://voicevox.github.io/voicevox_core/apis/python_api/autoapi/voicevox_core/index.html#voicevox_core.AudioQuery.from_accent_phrases">AudioQuery.from_accent_phrases</a>| aq
           -->|<a href="https://voicevox.github.io/voicevox_core/apis/python_api/autoapi/voicevox_core/blocking/index.html#voicevox_core.blocking.Synthesizer.synthesis">Synthesizer.synthesis</a>| wav

    linkStyle 0,1,2,3 font-family:monospace;
```

毎回これらの関数を経るのは大変なので、ショートハンドとなるAPIもあります。例えば[`Synthesizer.tts`]は日本語のテキストから直接音声を生成します。

[`Synthesizer.tts`]: https://voicevox.github.io/voicevox_core/apis/python_api/autoapi/voicevox_core/blocking/index.html#voicevox_core.blocking.Synthesizer.tts

```mermaid
flowchart TD;
    ja-txt[日本語のテキスト]
    ap-without-mora-data[音高・音素長<b>抜きの</b><br><a href="https://voicevox.github.io/voicevox_core/apis/python_api/autoapi/voicevox_core/index.html#voicevox_core.AccentPhrase">アクセント句</a>の列]
    ap-with-mora-data[音高・音素長<b>入りの</b><br><a href="https://voicevox.github.io/voicevox_core/apis/python_api/autoapi/voicevox_core/index.html#voicevox_core.AccentPhrase">アクセント句</a>の列]
    aq[<a href="https://voicevox.github.io/voicevox_core/apis/python_api/autoapi/voicevox_core/index.html#voicevox_core.AudioQuery">AudioQuery</a>]
    wav[WAV形式の音声]

    ja-txt ==>|<a href="https://voicevox.github.io/voicevox_core/apis/python_api/autoapi/voicevox_core/blocking/index.html#voicevox_core.blocking.Synthesizer.tts">Synthesizer.tts</a>| wav
    ja-txt ==>|<a href="https://voicevox.github.io/voicevox_core/apis/python_api/autoapi/voicevox_core/blocking/index.html#voicevox_core.blocking.Synthesizer.create_audio_query">Synthesizer.create_audio_query</a>| aq
    ja-txt ==>|<a href="https://voicevox.github.io/voicevox_core/apis/python_api/autoapi/voicevox_core/blocking/index.html#voicevox_core.blocking.Synthesizer.create_accent_phrases">Synthesizer.create_accent_phrases</a>| ap-with-mora-data
    ja-txt -->|<a href="https://voicevox.github.io/voicevox_core/apis/python_api/autoapi/voicevox_core/blocking/index.html#voicevox_core.blocking.OpenJtalk.analyze">OpenJtalk.analyze</a>| ap-without-mora-data
    ap-without-mora-data ==>|<a href="https://voicevox.github.io/voicevox_core/apis/python_api/autoapi/voicevox_core/blocking/index.html#voicevox_core.blocking.Synthesizer.replace_mora_data">Synthesizer.replace_mora_data</a>| ap-with-mora-data
    ap-without-mora-data -->|<ul><li><a href="https://voicevox.github.io/voicevox_core/apis/python_api/autoapi/voicevox_core/blocking/index.html#voicevox_core.blocking.Synthesizer.replace_phoneme_length">Synthesizer.replace_phoneme_length</a></li><li><a href="https://voicevox.github.io/voicevox_core/apis/python_api/autoapi/voicevox_core/blocking/index.html#voicevox_core.blocking.Synthesizer.replace_mora_pitch">Synthesizer.replace_mora_pitch</a></li><ul>| ap-with-mora-data
                         -->|<a href="https://voicevox.github.io/voicevox_core/apis/python_api/autoapi/voicevox_core/index.html#voicevox_core.AudioQuery.from_accent_phrases">AudioQuery.from_accent_phrases</a>| aq
                         -->|<a href="https://voicevox.github.io/voicevox_core/apis/python_api/autoapi/voicevox_core/blocking/index.html#voicevox_core.blocking.Synthesizer.synthesis">Synthesizer.synthesis</a>| wav

    %% 不可視の矢印を用いてできるだけ右側に「引っ張」る。
    ja-txt ~~~ aq
    ja-txt ~~~ wav

    linkStyle 0,1,2,3,4,5,6,7 font-family:monospace;
```
