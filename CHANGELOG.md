# Changelog

## [Unreleased]

<!--
### ソング

- [project-s] ピッチ輪郭推論を追加 ([#531])
- [project-s] モデルへの入力の形・データを修正 ([#732])
- [project-s] スタイルタイプの名称変更 ([#738])
- `StyleMeta::r#type`を追加し、トークという区分を実装に導入する ([#761])
- fix: fix up #761: JavaとPythonの`StyleType`を埋める ([#895])
- chore: [0.15] remove obsolete parts ([#896])
- Merge `0.15.5` ([#894])

[#732]: https://github.com/VOICEVOX/voicevox_core/pull/732
[#896]: https://github.com/VOICEVOX/voicevox_core/pull/896
[#894]: https://github.com/VOICEVOX/voicevox_core/pull/894

### ストリーミングAPI

- split decoder into spectrogram and vocoder without changing API ([#851])
- ストリーミングモードのdecodeを実装（precompute_renderとrender） ([#854])
- fix: Python APIとexample/python/run.pyの型付けを直す ([#864])
- fix compat breaking: revive workaround padding in decode() ([#867])
- feat!: `render`の引数の範囲指定部分を各言語の慣習に合わせる ([#879])
- feat!: decode.onnxを復活させる ([#918])

[#851]: https://github.com/VOICEVOX/voicevox_core/pull/851
[#854]: https://github.com/VOICEVOX/voicevox_core/pull/854
[#864]: https://github.com/VOICEVOX/voicevox_core/pull/864
[#867]: https://github.com/VOICEVOX/voicevox_core/pull/867
[#879]: https://github.com/VOICEVOX/voicevox_core/pull/879
[#918]: https://github.com/VOICEVOX/voicevox_core/pull/918

### もし`TextAnalyzer`機能を充実させた場合

- TextAnalyzer traitにstring->AccentPhraseModel[]を移動 ([#740] by [@eyr1n])。
- jlabel導入 ([#742] by [@phenylshima], [#750] by [@phenylshima])。
- feat!: Rust APIだけ`TextAnalyzer`をパブリックにする ([#919])

[#742]: https://github.com/VOICEVOX/voicevox_core/pull/742
[#750]: https://github.com/VOICEVOX/voicevox_core/pull/750
-->

### Added

- \[Rust,Python,Java\] 基本的なインターフェイスや特殊メソッドが、以下のように実装されます ([#1100])。

    - <details><summary>Rust API</summary>

      - `Debug` for
          - `AudioQuery`
          - `UserDictWordBuilder`
          - `{blocking,nonblocking}.onnxruntime.LoadOnce`
          - `{blocking,nonblocking}.VoiceModelFile`
          - `{blocking,nonblocking}.OpenJtalk`
          - `{blocking,nonblocking}.Synthesizer`
          - `{blocking,nonblocking}.synthesizer.*`
      - `PartialEq` for
          - `StyleMeta`
          - `AudioQuery`
          - `UserDictWord`
      - `{PartialOrd,Ord}` for
          - `AccelerationMode`
          - `UserDictWordType`
      - `Hash` for
          - `CharacterVersion`
          - `AccelerationMode`
      - `{AsRef,AsMut}` for `CharacterVersion`
      - `{UpperHex,LowerHex,Octal,Binary}` for `StyleId`
      - `Into<u32>` for `StyleId` (via `From`)
      </details>
    - <details><summary>Python API</summary>

      - `__repr__` for
          - `{blocking,asyncio}.VoiceModelFile`
          - `{blocking,asyncio}.Onnxruntime`
          - `{blocking,asyncio}.VoiceModelFile`
          - `{blocking,asyncio}.OpenJtalk`
          - `{blocking,asyncio}.UserDict`
      </details>
    - <details><summary>Java API</summary>

      - `Object.equals` for
          - `SupportedDevices`
          - `StyleMeta`
          - `CharacterMeta`
          - `Mora`
          - `AccentPhrase`
          - `AudioQuery`
      - `Cloneable` for
          - `SupportedDevices`
          - `StyleMeta`
          - `CharacterMeta`
          - `Mora`
      </details>

- 不要である[Oniguruma](https://github.com/kkos/oniguruma)のリンクをやめます ([#1082])。

- \[Rust\] Rust Analyzerが、C APIから参照する目的で[0.16.0-preview.0](#0160-preview0---2025-03-01-0900)の[#976]にて導入した`doc(alias)`に反応しないようになります ([#1099])。

- \[C\] `free`系と`delete`系の関数が、`free(3)`や`HeapFree`のようにヌルポインタを許容するようになります ([#1094])。

- \[C\] \[macOS\] :tada: GitHub ReleasesのXCFrameworkが、macOS向けのライブラリも同梱するようになります ([#1056])。

    ```diff
    -voicevox_core-ios-xcframework-cpu-{バージョン}.zip
    +voicevox_core-xcframework-cpu-{バージョン}.zip
     └── voicevox_core.xcframework
         ├── Info.plist
    +    ├── macos-arm64_x86_64/
         ├── ios-arm64/
         └── ios-arm64_x86_64-simulator/
    ```

    Changedの章で後述する通り、リリースの名前は変わります。

- \[Python\] exampleコードにはshebangが付き、filemodeも`100755` (`GIT_FILEMODE_BLOB_EXECUTABLE`)になります ([#1077])。

- \[Java\] \[Windows,macOS,Linux\] :tada: GitHub Releasesのjava\_packages.zipに、PC用のパッケージが追加されます ([#764])。

    ```diff
     java_packages.zip
     └── jp
         └── hiroshiba
             └── voicevoxcore
    +            ├── voicevoxcore/
                 └── voicevoxcore-android/
    ```

- \[ダウンローダー\] :tada: リトライ機構が導入され、デフォルトで4回のリトライを行うようになります ([#1098] by [@shuntia], [#1111])。

    ```console
      -t, --tries <NUMBER>
              ダウンロードにおける試行回数。'0'か'inf'で無限にリトライ [default: 5]
    ```

### Changed

- \[C\] \[macOS\] GitHub Releasesにおいてvoicevox\_core-**ios**-xcframework-cpu-{バージョン}.zipは、macOS版XCFrameworkの提供に伴ってvoicevox\_core-xcframework-cpu-{バージョン}.zipに改名されます ([#1056])。

### Removed

- \[Windows\] `windows-2019`がサポートから外れ、リリースは`windows-2022`で行われることになります ([#1096])。

    ただし、`windows-2022`でビルドしたバイナリであっても`windows-2019`相当の環境で動作すると考えられています。またVOICEVOX ONNX Runtimeが既に元々`windows-2022`でビルドされているため、通常の用途においては特に変わらないはずです。

### Fixed

- \[Rust\] Nightly Rustでビルドできない問題（[dtolnay/proc-macro2#497]）が発生したため、`proc-macro2`の依存がv1.0.95に上がります ([#1078])。
- \[Python\] ドキュメントの誤記が修正されます ([#1063])。
- \[Java\] \[Android\] GHAのUbuntuイメージ備え付けの`$ANDROID_NDK` (現時点ではバージョン27)を使ったリリースがされるようになります。これにより、[#1103]で報告されたAndroidビルドにおけるC++シンボルの問題が解決されます ([#1108])。
- \[Java\] Javaのファイナライザから中身のRustオブジェクトのデストラクトがされない問題が解決されます ([#1085])。
- \[ダウンローダー\] \[Windows\] GitHub Releasesにおいて、再び署名がされるようになります ([#1060])。

## [0.16.0] - 2025-03-29 (+09:00)

### Added

- 次の二つのドキュメントが追加され、APIドキュメント側からも言及されるようになります ([#1049])。
    - [docs/guide/user/languages.md](https://github.com/VOICEVOX/voicevox_core/blob/0.16.0/docs/guide/user/languages.md)
    - [docs/guide/user/serialization.md](https://github.com/VOICEVOX/voicevox_core/blob/0.16.0/docs/guide/user/serialization.md)
- [READMEの「事例紹介」](https://github.com/VOICEVOX/voicevox_core/blob/0.16.0/README.md#事例紹介)に、Swiftの事例として[VoicevoxCoreSwift](https://github.com/yamachu/VoicevoxCoreSwift)が追加されます ([#1055])。
- \[Rust\] APIドキュメントのトップにコード例が入ります ([#1016], [#1045])。
- \[ダウンローダー\] `models`のダウンロード元であるvoicevox\_vvmのバージョン範囲が、`>=0.1,<0.2`になります ([#1057])。

### Changed

- \[Rust\] \[BREAKING\] `UserDictWord`のコンストラクト方法がビルダースタイルになります ([#999])。
- \[Java\] \[BREAKING\] `jp.hiroshiba.voicevoxcore.GlobalInfo.SupportedDevices`は`voicevoxcore`直下に移動します ([#991])。
- \[Java\] \[BREAKING\] `UserDict`が扱う"UUID"はすべて、`String`ではなく`java.util.UUID`になります ([#1058])。
- \[Java\] \[Windows,macOS,Linux\] build.gradleに`javadoc.options.encoding = 'UTF-8'`が足されます ([#995])。

### Removed

- \[Java\] docs: APIドキュメントのポータルにおいて、Java APIのJavadocへの案内が一時的に消えます ([#1044])。

### Fixed

- \[Python\] `voicevox_core.{blocking,asyncio}`のクラスのインスタンスに対して、同時にアクセスしたときに`RuntimeError`が出る場合があった問題が解決されます ([#1041])。

- \[Python\] （返り値が`NoReturn`であるダミーの）`__new__`の型付けが直ります ([#1048])。

- \[Java\] \[Windows\] 同じ環境で二度起動しようとすると失敗する問題が修正されます ([#1043])。

     VOICEVOX CORE Java APIは、voicevox\_core\_java\_api.dllを`%TEMP%`直下に展開してそれをロードすることにより動いています。その動的ライブラリは[`File#deleteOnExit`](https://docs.oracle.com/javase/8/docs/api/java/io/File.html#deleteOnExit--)によってJVMの終了時に削除されるはずでしたが、Windowsの場合上手く消えないことがわかりました。そのためDLL展開時に、以前のものを[`REPLACE_EXISTING`](https://docs.oracle.com/javase/8/docs/api/java/nio/file/StandardCopyOption.html#REPLACE_EXISTING)で上書きすることで問題を解決します。

     voicevox\_core\_java\_api.dllは依然として`%TEMP%`下に残り続ける上に、VOICEVOX CORE Java APIの多重起動ができないことには変わらないことに注意する必要はあります。

- \[Java\] \[Windows,macOS,Linux\] 壊れていたKotlin exampleが直ります ([#994])。

- \[C\] \[Windows\] C++ exampleのREADME.mdの誤記が修正されます ([#1040] by [@nanae772])。

## [0.16.0-preview.1] - 2025-03-08 (+09:00)

### Added

- 書きかけの状態だった[docs/guide/user/usage.md](https://github.com/VOICEVOX/voicevox_core/blob/0.16.0-preview.1/docs/guide/user/usage.md)が書き上がります ([#1032])。
- readmeから「バージョン 0.15.4をご利用ください」の注意書きが削除されます ([#1035])。

### Changed

- \[Python\] \[BREAKING\] Pydanticが依存から外れ、`@pydantic.dataclasses.dataclass`のクラスはすべて素のdataclassになります。dataclassのシリアライズについては代替手段は用意されず、非推奨になります ([#1034])。
- \[ダウンローダー\] `models`のダウンロード元がvoicevox\_vvmの`0.0.1-preview.5` (= 今の[`0.1.0`](https://github.com/VOICEVOX/voicevox_vvm/releases/tag/0.1.0))になり、readmeおよび利用規約の文面が更新されます ([VOICEVOX/voicevox\_vvm#12], [VOICEVOX/voicevox\_vvm#14], [#1015])。

### Removed

- sample.vvmはGitHub Releasesに含まれなくなります ([#1033])。
- \[Linux\] \[BREAKING\] Ubuntu 20.04がサポート対象から外れ、バイナリのリリースはUbuntu 22.04で行われるようになります ([#1028])。

### Fixed

- \[ダウンローダー\] エラーメッセージの文面が修正されます ([#1030] by [@nanae772])。

## [0.16.0-preview.0] - 2025-03-01 (+09:00)

### Added

- :tada: Rust APIが利用できるようになります ([#702], [#745], [#740] by [@eyr1n], [#708], [#803], [#759], [#807], [#810], [#805], [#831], [#835], [#844], [#846], [#847], [#882], [#886], [#907], [#910], [#912], [#825], [#911], [#919], [#932], [#931], [#940], [#941], [#937], [#949], [#958], [#974], [#982], [#990], [#992], [#996], [#1002], [#1025] 他たくさん)。

    ```console
    ❯ cargo add voicevox_core --git https://github.com/VOICEVOX/voicevox_core.git --tag 0.16.0-preview.0 --features load-onnxruntime
    ```

    [mainブランチのAPIドキュメント](https://voicevox.github.io/voicevox_core/apis/rust_api/voicevox_core/)

- \[Python\] :tada: ブロッキングAPIを提供する`voicevox_core.blocking`モジュールが追加されます ([#702], [#706], [#992])。

    ```py
    from voicevox_core.blocking import Onnxruntime, OpenJtalk, Synthesizer, VoiceModelFile

    # …
    wav = synthesizer.tts("こんにちは", 0)
    ```

- 次のAPIが追加されます ([#1025])。

    - `AudioQuery::from_accent_phrases` (C API: `voicevox_audio_query_create_from_accent_phrases`)
    - `OpenJtalk::analyze` (C API: `voicevox_open_jtalk_rc_analyze`)

- `SpeakerMeta`および`StyleMeta`に、オプショナルな整数型フィールド`order`が追加されます ([#728])。

- `StyleMeta`に`type`というフィールドが追加されます ([#531], [#738], [#761], [#895], [#996])。

    取り得る値は`"talk" | "singing_teacher" | "frame_decode" | "sing"`です。ソング機能自体は今後[#1073]で行われる予定です。

- \[C,Python\] 次の関数の引数が不必要にUTF-8を要求することがなくなります ([#752])。

    - C
        - `voicevox_synthesizer_is_loaded_voice_model`
    - Python
        - `VoiceModel.is_loaded_voice_model`
        - `VoiceModel.from_path`

- \[Python,Java\] `Synthesizer`から`OpenJtalk`を得ることができるゲッターが追加されます ([#1025])。

- \[Python,Java\] `UserDict`の`load`と`store`が引数に取ることができるファイルパスの表現が広くなります ([#835])。

    Python APIでは`StrPath`相当になり、Java APIでは`java.io.File`と`java.nio.file.Path`のオーバーロードが追加されます。

- \[Python,Java\] exampleコードとそのドキュメントに細かい改善が入ります ([#881], [#986], [#992])。

- \[Python\] 引数の型が`Path | str`となっていた箇所は、一般的な慣習に合わせる形で`str | PathLike[str]`になります ([#753])。

- \[Python\] Pyright/Pylanceをサポートするようになります ([#719])。

- \[C\] `VoicevoxSynthesizer`などのオブジェクトに対する`…_delete`が、どのタイミングで行っても安全になります ([#849], [#862])。

    - "delete"時に対象オブジェクトに対するアクセスがあった場合、アクセスが終わるまで待つようになります。
    - 次の操作が未定義動作ではなくなります。ただし未定義動作ではないだけで明示的にクラッシュするため、起きないように依然として注意する必要があります。
        - "delete"後に他の通常のメソッド関数の利用を試みる
        - "delete"後に"delete"を試みる
        - そもそもオブジェクトとして変なダングリングポインタが渡される
        - ヌルポインタが渡される (注: [#1094]にて許容される予定です)

- \[C\] リリース内容物にLICENSEファイルが追加されます ([#965])。

- \[Python\] :tada: 推論を行うAPIにオプション引数`cancellable`が追加されます ([#889], [#1024], [#903], [#992])。

    `True`にすると[タスクとしてキャンセル](https://docs.python.org/3.11/library/asyncio-task.html#task-cancellation)できるようになります。

    デフォルトでキャンセル可能ではない理由は、ドキュメントにも書いてありますがキャンセル可能にすると（キャンセルを行わない場合でも）[ハングする危険性がある](https://github.com/VOICEVOX/voicevox_core/issues/968)からです。ご注意ください。

- \[Python\] wheelは`Metadata-Version: 2.4`になり、またライセンス情報とreadmeが含まれるようになります ([#947], [#949], [#959])。

- \[ダウンローダー\] 対象外の`<TARGET>`を見に行かないようになります ([#939])。

    これまではダウンロード対象外であっても、不必要にリポジトリを見にいくようになってました。

- TODO: readme関連
    - [docs] ユーザーガイドを追加 ([#699])。
    - [docs] ドキュメント整理（ユーザーガイドをリンク、VVMのリンク追加、利用規約があることを案内） ([#707])。
    - Update jump-to version on README ([#824] by [@cm-ayf]).
    - chore: READMEからvoicevox.github.io/voicevox_core/apisにリンク ([#838])
    - feat(docs): docs/を整理する ([#863])
    - docs: ダウンローダー周りの記述を更新 ([#945])
    - docs(fix): readmeの古い記述を更新 ([#1019])
        - 0.15.0-preview.16からのfeatも含まれる
    - docs: readmeのダイエット ([#1021])
        - featのはず
- TODO: APIドキュメント関連
    - chore: voicevox.github.io/voicevox_core/apis内のリンクを置き換え ([#837])
    - chore: READMEからvoicevox.github.io/voicevox_core/apisにリンク ([#838])
    - fix(docs): `SpeakerMeta.{speaker_uuid,version}`が逆だった ([#935])
        - これはfix
    - feat!: "話者" ("speaker") → "キャラクター" ("character") ([#943])
    - docs: [Python] 型エイリアス系へのリンクについてワークアラウンド ([#952])
    - docs: [Python] Sphinxをv8に上げ、extension達もアップデート ([#953])
    - docs: [C] 各アイテムからRust APIにリンクを張る ([#976])
    - docs: 軽く解決可能なTODOとFIXMEを解消 ([#992])
    - docs: APIドキュメントの`{Character,Style}Meta`周りの記述を統一 ([#996])
        - 0.15.0-preview.16からのfixも含まれる
    - docs: "ダウンローダーがダウンロードするもの"の節を追加 ([#1023])
        - feat
    - feat: いくつかのAPIを露出し、「テキスト音声合成の流れ」を明確に ([#1025])
        - feat

### Changed

- \[BREAKING\] :tada: VOICEVOX COREは完全にMIT Licenseになり、代わりにプロプライエタリ部分はONNX Runtime側に移ります ([#913], [#825], [#965], [#973], [#979], [#1019])。

    御自身で手を加えたVOICEVOX COREをそのまま実行できるようになります。

    製品版VVMを読み込む際は、ONNX Runtimeの代わりに**VOICEVOX** ONNX Runtimeというライブラリが必要になります。VOICEVOX ONNX Runtimeは、ダウンローダーにて`onnxruntime`としてダウンロードできます。

    ```console
    ❯ ./download --only onnxruntime --onnxruntime-version voicevox_onnxruntime-1.17.3
    ```

- \[BREAKING\] :tada: (VOICEVOX) ONNX Runtimeを動的リンクすることは基本的になくなり、代わりに`dlopen`/`LoadLibraryExW`でロードするようになります。ロードは`Onnxruntime`型から行う形になります ([#725], [#802], [#806], [#810], [#822], [#860], [#876], [#898], [#921], [#911], [#933], [#992], [#1003], [#1019])。

    これにより、VOICEVOX COREの起動後に(VOICEVOX) ONNX Runtimeを探して読み込むことができるようになりました。ただし、iOS版のリリースにおいてのみ従来の動的リンクの形を継続します。

    またこれに伴い:

    - C APIでは、LinuxとmacOS用のrpath設定が削除されます。
    - Python APIはmanylinuxに対応するようになり、wheel名の"linux"は"manylinux_{glibcのバージョン}"になります。また、カレントディレクトリ下の動的ライブラリを自動で読み込む機能は無くなります。

- \[BREAKING\] VOICEVOX CORE自体からはCPU版/GPU版という区分は無くなり、GPU違いのリリースについては完全に(VOICEVOX) ONNX Runtimeに委ねる形になります ([#802], [#810])。

- \[BREAKING\] このリポジトリのGitHub ReleasesはONNX Runtimeを含まなくなります。Java APIの依存からはcom.microsoft.onnxruntime/onnxruntime{,\_gpu}が消えます。 ([#810])。

    ダウンローダーは[VOICEVOX/onnxruntime-builder](https://github.com/VOICEVOX/onnxruntime-builder)から直接(VOICEVOX) ONNX Runtimeをダウンロードするようになります。

- \[BREAKING\] VVMの形式が変わり、[0.15.0-preview.16](#0150-preview16---2023-12-01-0900)までのVVMは利用できなくなります ([#794], [#795], [#796], [#825])。

- \[BREAKING\] 製品版VVMは、このリポジトリのGitHub Releasesには置かれなくなります ([#928], [#964], [#1020] by [@nanae772])。

    [VOICEVOX/voicevox\_vvm](https://github.com/VOICEVOX/voicevox_vvm)に置かれるようになり、ダウンローダーはそこからダウンロードします。なお、VOICEVOX/voicevox\_fat\_resourceは[リポジトリごと削除されました](https://github.com/VOICEVOX/voicevox_fat_resource)。

- \[BREAKING\] `acceleration_mode`を`GPU`または`AUTO`（デフォルト）にしたときの挙動が変わります ([#810])。

    `Synthesizer`のコンストラクトの時点でGPUの簡易的なチェックを行うことで、適切なGPUの種類が選択されるようになります。チェックがすべて失敗した場合、`GPU`であればエラー、`AUTO`であればCPUにフォールバックとなります。

- \[BREAKING\] `AudioQuery`および`UserDictWord`のJSON表現はVOICEVOX ENGINEと同じになります ([#946], [#1014])。

    これにより、VOICEVOX ENGINEとVOICEVOX COREとで同じ`AudioQuery`と`UserDictWord`が使い回せるようになります。Python APIおよびJava APIにおける、クラスの形には影響しません。

    ```json
    {
      "accent_phrases": […],
      "speedScale": 1.0,
      "pitchScale": 0.0,
      "intonationScale": 1.0,
      "volumeScale": 1.0,
      "prePhonemeLength": 0.1,
      "postPhonemeLength": 0.1,
      "outputSamplingRate": 24000,
      "outputStereo": false
    }
    ```

- \[BREAKING\] `VoiceModelId`は、VVMに固有のUUIDになります ([#796])。

- \[BREAKING\] `InferenceFailed`エラーは `RunModel`エラーになります ([#823]).

- \[BREAKING\] `ExtractFullContextLabel`エラーは`AnalyzeText`エラーになります ([#919])。

- \[BREAKING\] `UserDictWord`の`accent_type`はオプショナルではなくなります ([#1002])。

    VOICEVOX ENGINEに合わせる形です。

- \[BREAKING\] `UserDictWord`の`priority`のデフォルトが`0`から`5`に変わります ([#1002])。

    Python API、Java API、VOICEVOX ENGINEに合わせる形です。

- `Synthesizer::unload_voice_model`と`UserDict::remove_word`における削除後の要素の順序が変わります ([#846])。

    例えば`[a, b, c, d, e]`のようなキーの並びから`b`を削除したときに、順序を保って`[a, c, d, e]`になります。以前までは`[a, e, c, d]`になってました。

- \[C\] \[BREAKING\] 次の`VoicevoxVoiceModelFile`のゲッターに位置付けられる関数が、ゲッターではなくなります ([#850])。

    - `voicevox_voice_model_file_id`

        `uint8_t (*output_voice_model_id)[16]`に吐き出すように。

    - `voicevox_voice_model_file_get_metas_json`

        `voicevox_voice_model_file_create_metas_json`に改名。

- \[C\] \[BREAKING\] リリース内容物において、動的ライブラリはlib/に、ヘッダはinclude/に入るようになります ([#954], [#967], [#980])。

    ```
    ├── include
    │   └── voicevox_core.h
    ├── lib
    │   ├── voicevox_core.dll
    │   └── voicevox_core.lib
    ├── LICENSE
    ├── README.txt
    └── VERSION
    ```

- \[Python,Java\] \[BREAKING\] `SpeakerMeta`は<code>**Character**Meta</code>に、`StyleVersion`は<code>**Character**Meta</code>に改名されます ([#931], [#943], [#996])。

- \[Python\] \[BREAKING\] ブロックングAPIの実装に伴い、`Synthesizer`, `OpenJtalk`, `VoiceModel`, `UserDict`は`voicevox_core.asyncio`モジュール下に移動します ([#706])。

- \[Python\] \[BREAKING\] `Enum`だったクラスはすべて`Literal`と、実質的なボトム型`_Reserved`の合併型になります ([#950], [#957])。

    ```diff
    -class AccelerationMode(str, Enum):
    -    AUTO = "AUTO"
    -    CPU = "CPU"
    -    GPU = "GPU"
    +AccelerationMode: TypeAlias = Literal["AUTO", "CPU", "GPU"] | _Reserved
    ```

    `_Reserved`の存在により、型チェックにおいて`match`での網羅はできなくなります。

- \[Python\] \[BREAKING\] `Synthesizer.audio_query`は、C APIとJava APIに合わせる形で`create_audio_query`に改名されます ([#882])。

- \[Python\] \[BREAKING\] `UserDict.words`は`UserDict.to_dict`に改名されます ([#977])。

- \[Python\] \[BREAKING\] `Synthesizer.metas`と`UserDict.words`は`@property`ではなく普通のメソッドになります ([#914])。

- \[Python\] \[BREAKING\] `UserDictWord`へのPydanticは非サポートになります。またdataclassとして`frozen`になり、コンストラクタ時点で各種バリデートが行われるようになります ([#1014])。

- \[Python\] \[BREAKING\] デフォルト引数の前には一律で`*,`が挟まれるようになります ([#998])。

- \[Java\] \[BREAKING\] `Synthesizer`, `OpenJtalk`, `VoiceModelFile`, `UserDict`は`voicevoxcore.blocking`パッケージの下に移ります。それに伴い、いくつかのクラスは`voicevoxcore`パッケージの直下に置かれるようになります ([#861])。

    - `voicevoxcore.{Synthesizer. => }AccelerationMode`
    - `voicevoxcore.{VoiceModelFile. => }SpeakerMeta`
    - `voicevoxcore.{VoiceModelFile. => }StyleMeta`
    - `voicevoxcore.{UserDict.Word => UserDictWord}`

    (`Synthesizer`, `VoiceModelFile`, `UserDict`自体は`voicevoxcore.blocking`下に移動)

- \[Java\] \[BREAKING\] `AccelerationMode`と`UserDictWord.Type`はenumではなくなり、`switch`での網羅ができなくなります ([#955])。

    それぞれの値自体はそのままの名前で`public static final`な定数として定義されているので、引き続きそのまま利用可能です。

    ```java
    var mode = AccelerationMode.AUTO;
    ```

- \[Java\] \[BREAKING\] ビルダーパターンメソッドの締めの`execute`は`perform`に改名されます ([#911])。

- \[ダウンローダー\] \[BREAKING\] `onnxruntime`（新規追加）および`models`のダウンロードの際、利用規約への同意が求められるようになります ([#928], [#983], [#989], [#1006], [#1011])。

- \[ダウンローダー\] \[BREAKING\] `<TARGET>`のうち`core`は`c-api`に改名され、それに伴い`-v, --version`も`--c-api-version`、`--core-repo`も`--c-api-repo`に改名されます ([#942], [#1019])。

- \[ダウンローダー\] \[BREAKING\] `<TARGET>`ごとにディレクトリが切られるようになります ([#944], [#969])。

    ```console
          --only <TARGET>...
              ダウンロード対象を限定する [possible values: c-api, onnxruntime, additional-libraries, models, dict]
          --exclude <TARGET>...
              ダウンロード対象を除外する [possible values: c-api, onnxruntime, additional-libraries, models, dict]
    ```

    ```
    voicevox_core
    ├── c_api/
    ├── onnxruntime/
    ├── additional_libraries/
    ├── models/
    └── dict/
    ```

- \[ダウンローダー\] \[BREAKING\] `models`において、上記のようにmodels/下に置かれるようになった他に次のような変更があります ([#928], [#989])。

    - VVM自体はvvm/というディレクトリに入る形になります。
    - README.mdはREADME.txtとして置かれるようになります。
    - [0.15.0-preview.16](#0150-preview16---2023-12-01-0900)まで含まれていたmetas.jsonは無くなります。

- \[BREAKING\] `VoiceModel`は`VoiceModelFile`になり、ファイルディスクリプタを保持する形になります。コンストラクタの名前は"from\_path"から"open"になり、Python APIとJava APIではクローズ可能になります ([#832], [#937], [#993])。

    クローズ (`__{,a}{enter,exit}__`/`java.io.Closeable`)の挙動については、詳しくはAPI ドキュメントをご覧ください。

### Deprecated

- \[Python,Java\] PydanticおよびGSONは廃止予定になります ([#985])。

    現段階においては代替手段は無く、シリアライズ自体が推奨されない状態になっています。

    GSONについてはJacksonへの移行が予定されています ([#984])。

    補足: Pydanticについては[0.16.0-preview.1](#0160-preview1---2025-03-08-0900)で消されます。

### Removed

- \[macOS\] macOS 11およびmacOS 12がサポート範囲から外れ、バイナリのリリースはmacOS 13で行われるようになります ([#801], [#884])。

- \[Python,Java\] \[BREAKING\] `SupportedDevices`のデシアライズ（JSON → `SupportedDevices`の変換）ができなくなります。Python APIにおいてはコンストラクトもできなくなります ([#958])。

- \[Python\] \[BREAKING\] Pythonのバージョンが≧3.10に引き上げられます ([#915], [#926], [#927])。

    Python 3.10以降では、[asyncioランタイム終了時にクラッシュする問題](https://github.com/VOICEVOX/voicevox_core/issues/873)が発生しなくなります。

- \[Java\] \[BREAKING\] `UserDict.Word`改め`UserDictWord`には、GSONによるシリアライズは使えなくなります ([#1014])。

### Fixed

- TODO: 非同期周りの改善

    - fix: 非同期関連のtodoとfixmeを解消 ([#868])
    - #830 の設計を`UserDict`にも ([#834])
        - `async-fs`の導入 (changelogに書くには微妙すぎる?)

- "Added"の章で述べた`SpeakerMeta::order`により、製品版VVMにおいて`metas`の出力が適切にソートされるようになります ([#728])。

    これにより、キャラクター/スタイルの順番がバージョン0.14およびVOICEVOX ENGINEのように整います。

- 空の`UserDict`を`use_user_dict`したときにクラッシュする問題が修正されます ([#733])。

- `VoiceModelFile::open` (旧`VoiceModel::from_path`)の実行時点で、ある程度の中身のバリデートがされるようになります ([#830])。

- \[C\] `voicevox_user_dict_add_word`がスタックを破壊してしまう問題が修正されます ([#800])。

- \[C\] \[iOS\] XCFrameworkへのdylibの入れかたが誤っていたために[App Storeへの申請が通らない](https://github.com/VOICEVOX/voicevox_core/issues/715)状態だったため、入れかたを変えます ([VOICEVOX/onnxruntime-builder#25] by [@nekomimimi], [#723] by [@nekomimimi])。

- \[C\] \[iOS\] clang++ 15.0.0でSIM向けビルドが失敗する問題が解決されます ([#720] by [@nekomimimi])。

- \[Python\] `StyleMeta`が`voicevox_core`モジュール直下に置かれるようになります ([#930])。

- \[Python\] 型定義において呼べないはずのコンストラクタが呼べることになってしまってたため、ダミーとなる`def __new__(cls, *args, **kwargs) -> NoReturn`を定義することで解決します。エラーメッセージも改善されます ([#988], [#997])。

### Security

- VOICEVOX COREおよびダウンローダーに影響するものだったのかどうかはわかりませんが、以下の脆弱性登録の影響を受けないように依存ライブラリがアップデートされます ([#856], [#887], [#890])。

    - [RUSTSEC-2024-0332](https://rustsec.org/advisories/RUSTSEC-2024-0332)
    - [RUSTSEC-2024-0336](https://rustsec.org/advisories/RUSTSEC-2024-0336)
    - [RUSTSEC-2024-0402](https://rustsec.org/advisories/RUSTSEC-2024-0402)
    - [RUSTSEC-2024-0404](https://rustsec.org/advisories/RUSTSEC-2024-0404)
    - [RUSTSEC-2024-0421](https://rustsec.org/advisories/RUSTSEC-2024-0421)

## [0.15.0-preview.16] - 2023-12-01 (+09:00)

### Added

- \[Python\] `StyleId`, `StyleVersion`, `VoiceModelId`が`NewType`として導入されます ([#678])。

    関数の引数としては`style_id: StyleId | int`のようになり、元の型は引き続き利用できます。

- \[Python\] 次のdataclassのフィールドが、VOICEVOX ENGINEのようにオプショナルになります ([#696])。

    - `Mora.consonant: Optional[str]`
    - `Mora.consonant_length: Optional[str]`
    - `AccentPhrase.pause_mora: Optional[Mora]`
    - `AccentPhrase.is_interrogative: bool`
    - `AudioQuery.kana: Optional[str]`

- TODO: API docs関連
    - Androidでの注意を追加 ([#682])。

### Changed

- \[Python\] \[BREAKING\] 次のメソッドがasync化されます ([#667])。

    - `UserDict.load`
    - `UserDict.save`
    - `OpenJtalk.__new__` (利用できなくなり、代わりに`OpenJtalk.new`が追加されます)
    - `OpenJtalk.use_user_dict`

- \[Python\] \[BREAKING\] Pydanticがv2になります ([#695])。

- TODO: この`withSourcesJar`、何…?
    - Androidでの注意を追加 ([#682])。

### Fixed

- \[Python\] 音声合成の処理がasyncioのランタイムを阻害しないようになります ([#692])。

- TODO: これだと壊れるんだっけ…?
    - UserDict.to_mecab_formatが2重に改行していたのを修正 ([#684])。

### Non notable

- TODO: Rust APIの布石
    - ONNX Runtimeとモデルのシグネチャを隔離する ([#675])。
    - IOが発生するメソッドをすべてasync化する ([#667])。
    - 音声合成の処理を丸ごとスレッド分離して実行する ([#692])。
    - `OpenJtalk`を`Synthesizer<OpenJtalk> | Synthesizer<()>`として持つ ([#694])。

## [0.15.0-preview.15] - 2023-11-13 (+09:00)

### Added

- \[Python,Java\] `Synthesizer`が不要に排他制御されていたのが解消されます ([#666])。
- \[Java\] `Synthesizer#{getMetas,isGpuMode}`および、バージョン情報とデバイス情報が取得できる`GlobalInfo`クラスが追加されます ([#673])。
- `InferenceFailed`エラーが、ONNX Runtimeからの情報をきちんと持つようになります ([#668])。

- TODO: readme関連
    - READMEのドキュメントの順番を整理 ([#661])。
- TODO: example関連
    - Python版exampleで、asyncやawaitは必須であることをコメントで書いておく ([#663])。
- TODO: API docs関連
    - Java APIを色々改善 ([#673])。

### Changed

- \[C,Python\] \[BREAKING\] `Synthesizer`および`OpenJtalk`の`new_with_initialize`は`new`にリネームされます ([#669])。
- \[Python\] \[BREAKING\] `Synthesizer.new`は無くなり、`__new__`からコンストラクトできるようになります ([#671])。

### Non notable

- TODO: Pythonのsdist
    - Maturin, PyO3, pyo3-asyncio, pyo3-logをアップデート ([#664])。
- TODO: Rust API
    - "new_with_initialize" → "new" ([#669])。

## [0.15.0-preview.14] - 2023-10-27 (+09:00)

### Added

- \[ダウンローダー\] :tada: ダウンロード対象を限定および除外するオプションが追加されます ([#647])。

    `--only <TARGET>...`で限定、`--exclude <TARGET>...`で除外ができます。`--min`は`--only core`のエイリアスになります。

- \[Python,Java\] エラーの文脈が例外チェーンとしてくっつくようになりました ([#640])。

### Changed

- \[Python,Java\] `VoicevoxError`/`VoicevoxException`は解体され、個別のエラーになります ([#640])。

### Fixed

- \[Java\] MUTF-8である`String`の内容を誤ってUTF-8として認識してしまっていた問題が解決されます ([#654])。

### Non notable

- TODO: Rust API
    - `workspace.resolver`を設定 ([#646])。
    - 不要な依存を削除 ([#656])。

## [0.15.0-preview.13] - 2023-10-14 (+09:00)

### Fixed

- \[ダウンローダー\] Windows版のビルドの問題が解決され、リリースされるようになりました ([#643])。

## [0.15.0-preview.12] - 2023-10-14 (+09:00)

Windows版ダウンローダーのビルドに失敗しています。

### Added

- :tada: Android向けにJava APIが追加されます ([#558], [#611], [#612], [#621])。

    ```java
    var wav = synthesizer.tts("こんにちは", 0).execute();
    ```

    ~/.m2/repository/の内容をZIPにしたものがjava\_package.zipとしてリリースされます。

- \[ダウンローダー\] リポジトリ指定機能が追加されます ([#641])。

    `--core-repo <REPOSITORY>`でvoicevox\_core（C API）の、`--additional-libraries-repo <REPOSITORY>`でvoicevox\_additional\_librariesのリポジトリを指定できます。

    ```console
    ❯ download --core-repo ${fork先}/voicevox_core --additional-libraries-repo ${fork先}/voicevox_additional_libraries
    ```

- TODO: readmeの改善
    - [docs] Rust以外の１つの言語でのコア機能追加実装はしない方針であることを明記 ([#632])。

### Changed

- \[BREAKING\] VVMはC APIのリリースに同梱される形でしたが、独立してmodel-{version}.zipとしてリリースされるようになります ([#603])。

### Fixed

- \[C\] 不正な`delete`および`json_free`に対するセーフティネットのメッセージが改善されます ([#625])。

## [0.15.0-preview.11] - 2023-10-08 (+09:00)

### Fixed

- TODO: 内容物のfix?
    - リソースのバージョンを更新 ([#630])

## [0.15.0-preview.10] - 2023-10-07 (+09:00)

### Added

- TODO: APIドキュメントの改善
    - Sphinxをv6に上げる ([#626])。

### Changed

- `kana: bool`をやめ、"_from_kana"を復活させる ([#577])。
- `InvalidStyleId`, `InvalidModelId`, `UnknownWord`を`…NotFound`にする ([#622])。
- `UnloadedModel` → `ModelNotFound` ([#623])。
- エラーメッセージにおけるcontextとsourceを明確に区分する ([#624])。

## [0.15.0-preview.9] - 2023-09-18 (+09:00)

### Added

- \[Rust版ダウンローダー\] helpの表示が改善されます ([#604])。
- \[C\] 引数の`VoicevoxUserDictWord*`はunalignedであってもよくなります ([#601])。
- \[Python\] `__version__`が追加されます ([#597])。
    - TODO: これを"Added"とするならば、`__version__`を実装したときまで遡って探す必要がある
- TODO: readmeの改善
    - C# の参考実装のリポジトリをREADMEに記載する ([#590])。
- TODO: exampleの改善
    - 他言語のlintのWorkflowを追加 ([#598])。
    - Code: blackでフォーマット ([#613])。
    - example下の"speaker_id"を、"style_id"に直す ([#584] by [@weweweok])。

### Changed

- \[C\] エラーの表示は`ERROR`レベルのログとしてなされるようになります ([#600])。
- \[Rust版ダウンローダー\] \[BREAKING\] `--min`と`--additional-libraries-version`同時使用は無意味であるため、禁止されます ([#605])。

### Removed

- \[BREAKING\] `load_all_models`が廃止されます ([#587])。

    [0.15.0-preview.5]以降においても`${dllの場所}/model/`もしくは`$VV_MODELS_ROOT_DIR`下のVVMを一括で読む機能として残っていましたが、混乱を招くだけと判断して削除されることとなりました。

- \[BREAKING\] Bash版ダウンローダーとPowerShell版ダウンローダーは削除されます ([#602])。

    Rust版をお使いください。

### Fixed

- \[C\] ログ出力においてANSI escape sequenceを出すかどうかの判定を改善しました ([#616])。

    従来は環境変数のみで判定していましたが、これからはstderrがTTYかどうかを見て、必要なら`ENABLE_VIRTUAL_TERMINAL_PROCESSING`を有効化するようになります。

### Non notable

- TODO: Rust API?
    - Rust APIが公開するエラーの種類を`ErrorKind`として表現する ([#589])。

## [0.15.0-preview.8] - 2023-08-26 (+09:00)

### Fixed

- 各ライブラリがきちんとリリースされるようになりました ([#586])。

## [0.15.0-preview.7] - 2023-08-24 (+09:00)

各ライブラリのビルドが不可能な状態に陥り、ダウンローダーだけがリリースされています。コミットとしては[0.15.0-preview.6]と同一です。

## [0.15.0-preview.6] - 2023-08-24 (+09:00)

### Added

- TODO: mutabilityとasyncnessを仕上げる ([#553])。
- \[Python\] `Synthesizer`に`__enter__`と`__close__`が実装されます ([#555])。
- \[C\] \[iOS\] XCFrameworkにmodulemapが入るようになります ([#579] by [@fuziki])。

### Changed

- \[C\] \[BREAKING\] C APIの名前を少し変更 ([#576])。
- \[C\] \[BREAKING\] `voicevox_synthesizer_audio_query`は`voicevox_synthesizer_create_audio_query`にリネームされます ([#576])。
- \[C\] \[BREAKING\] 定数化されたものを関数へ戻します ([#557] by [@shigobu])。

### Fixed

- TODO: Pythonドキュメント周りの色々を修正 ([#570])。
- TODO: voicevox_json_freeの対象が漏れていたことの修正 ([#571])。
- TODO: VoiceModelのget_all_modelsがvvm以外のファイルも読み込もうとしてクラッシュすることの修正 ([#574])。
- TODO: C-APIのnew_with_initializeで初期化した場合、metas jsonが空になってしまうことの修正 ([#575])。

### Non notable

- TODO: Rust APIとして…?
    - RustのdoctestをCI ([#573])。
    - `VoicevoxResultCode`をC APIに移動 ([#580])。

## [0.15.0-preview.5] - 2023-08-06 (+09:00)

### Added

- :tada: ユーザー辞書機能が使えるようになります ([#538], [#546])。
- TODO: ドキュメント改善
    - ドキュメントを刷新する ([#532])
    - Add: IGNORE_PREFIXオプションを追加 ([#565])

### Changed

- TODO: project-vvm-async-api ([#497])
    - 新クラス設計API ([#370])
    - [project-vvm-async-api] ドキュメントの表記ゆれを解消 ([#501])
    - [project-vvm-async-api] `voicevox_{,synthesizer_}is_loaded_voice_model` ([#523])
    - [project-vvm-async-api] 工事中の案内を書く ([#542])
    - [project-vvm-async-api] C/Python APIクレート側のバージョンを出す ([#507])
    - [project-vvm-async-api] `get_supported_devices_json`をfallibleに ([#502])
    - [project-vvm-async-api] いくつかのC関数を定数にする ([#503])
    - [project-vvm-async-api] ZSTにポインタキャストして提供するのをやめる ([#512])
    - [project-vvm-async-api] `extern "C"`の生ポインタをABI互換のに置き換え ([#514])
    - [project-vvm-async-api] "buffer"をRustの世界で保持し続ける ([#525])
    - [project-vvm-async-api] `output_`系引数がunalignedであることを許す ([#534])
    - [project-vvm-async-api] whlに"modelディレクトリ"を埋め込むのをやめる ([#522])
    - [project-vvm-async-api] Fix up #500 ([#521])
    - [project-vvm-async-api] Fix up #534 ([#535])
    - 製品版VVMを使うようにする ([#569])
    - styleIdとsession.runに渡す数値が異なっているVVMでも音声合成できるようにする ([#551])

### Deprecated

### Removed

### Fixed

### Security

### Non notable

- [project-vvm-async-api] mainをマージする ([#516])
- [project-vvm-async-api] mainをsquashせずにマージする ([#520])
- [project-vvm-async-api] mainをマージする ([#536])

## [0.15.0-preview.4] - 2023-06-21 (+09:00)

### Added

- TODO: readmeとexampleの改善
    - 事例紹介にvoicevoxcore.goを追加 ([#498] by [@sh1ma])
    - Fix up #421 ([#494])
    - Pythonコードをリファクタ ([#495])
    - READMEのスペースが足りてなかった ([#511])
- \[C\] :tada: iOS向けXCFrameworkがリリースに含まれるようになります ([#485] by [@HyodaKazuaki])。
- \[C\] 知らない文字列、既知の静的領域の文字列、解放済みの文字列への`json_free`は明示的に拒否されるようになります ([#500])。
- \[C\] ヘッダに[cbindgen](https://docs.rs/crate/cbindgen)のバージョンが記載されるようになります ([#519])。
- \[C\] ヘッダにおける変な空行が削除されます ([#518])。
- \[Python\] Rustのパニックが発生したときの挙動が「プロセスのabort」から、「`pyo3_runtime.PanicException`の発生」に変わります ([#505])。

## [0.15.0-preview.3] - 2023-05-18 (+09:00)

### Added

- :tada: 音素の長さ、もしくは音高の再生成ができるようになります ([#479], [#483])。

    VOICEVOX ENGINEの`/mora_{length,pitch,data}`にあたります。

- `AudioQuery`ではない、`accent_phrases`のみの生成ができるようになります ([#479], [#483])。

    VOICEVOX ENGINEの`/accent_phrases`にあたります。

- `AudioQuery`の`kana`が、VOICEVOX ENGINEと同様に省略可能になります ([#486], [#487])。

- APIドキュメントが改善されます ([#438])。

    - テキストの文字コードはUTF8だと案内

- TODO: readmeとexampleの改善

    - Goサンプルコードを追加 ([#455])
    - voicevoxcore4s(Scala FFI Wrapper)を事例紹介に追加 ([#429])
    - Flutter 向け FFI ラッパーを事例紹介に追加 ([#458])
    - READMEにDiscordへの案内などを追加 ([#463])
    - ダウンローダーをスクリプト版からrust版を使うよう案内 ([#439])
    - 0.13工事中の表記を消す ([#404])
    - example/pythonのloggingを改善 ([#481])
    - example/pyo3 を利用しやすく修正 ([#419]) ([#475])
    - wheelを利用したexampleをわかりやすく ([#421])
    - init.pyに__all__を追加 ([#415])
    - Windows c++サンプル修正 ([#420])
    - python (FFI) example を削除 ([#432])

- \[C\] :tada: Androidをターゲットとしたビルドが追加されます ([#444] by [@char5742], [#450], [#452] by [@char5742], [#473])。

- \[C\] :tada: iOSをターゲットとしたビルドが追加されます ([#471] by [@HyodaKazuaki])。

- \[C\] アロケーションの回数を抑えるパフォーマンス改善が入ります ([#392], [#478])。

    TODO: フールプルーフ機構がこのあたりから入ってなかったか？要確認

- \[Rust版ダウンローダー\] download-windows-x64.exeはコード署名されるようになります ([#412])。

### Changed

- \[C\] ログの時刻がローカル時刻になります ([#400], [#434])。
- \[Rust版ダウンローダー\] \[BREAKING\] リリースの`download-{linux,osx}-aarch64`は`…-arm64`に改名されます ([#416])。

### Fixed

- `kana`オプションを有効化したときに、音素の流さと音高が未設定になってしまう問題が修正されます ([#407])。

[Unreleased]: https://github.com/VOICEVOX/voicevox_core/compare/0.16.0...HEAD
[0.16.0]: https://github.com/VOICEVOX/voicevox_core/compare/0.16.0-preview.1...0.16.0
[0.16.0-preview.1]: https://github.com/VOICEVOX/voicevox_core/compare/0.16.0-preview.0...0.16.0-preview.1
[0.16.0-preview.0]: https://github.com/VOICEVOX/voicevox_core/compare/0.15.0-preview.16...0.16.0-preview.0
[0.15.0-preview.16]: https://github.com/VOICEVOX/voicevox_core/compare/0.15.0-preview.15...0.15.0-preview.16
[0.15.0-preview.15]: https://github.com/VOICEVOX/voicevox_core/compare/0.15.0-preview.14...0.15.0-preview.15
[0.15.0-preview.14]: https://github.com/VOICEVOX/voicevox_core/compare/0.15.0-preview.13...0.15.0-preview.14
[0.15.0-preview.13]: https://github.com/VOICEVOX/voicevox_core/compare/0.15.0-preview.12...0.15.0-preview.13
[0.15.0-preview.12]: https://github.com/VOICEVOX/voicevox_core/compare/0.15.0-preview.11...0.15.0-preview.12
[0.15.0-preview.11]: https://github.com/VOICEVOX/voicevox_core/compare/0.15.0-preview.10...0.15.0-preview.11
[0.15.0-preview.10]: https://github.com/VOICEVOX/voicevox_core/compare/0.15.0-preview.9...0.15.0-preview.10
[0.15.0-preview.9]: https://github.com/VOICEVOX/voicevox_core/compare/0.15.0-preview.8...0.15.0-preview.9
[0.15.0-preview.8]: https://github.com/VOICEVOX/voicevox_core/compare/0.15.0-preview.7...0.15.0-preview.8
[0.15.0-preview.7]: https://github.com/VOICEVOX/voicevox_core/compare/0.15.0-preview.6...0.15.0-preview.7
[0.15.0-preview.6]: https://github.com/VOICEVOX/voicevox_core/compare/0.15.0-preview.5...0.15.0-preview.6
[0.15.0-preview.5]: https://github.com/VOICEVOX/voicevox_core/compare/0.15.0-preview.4...0.15.0-preview.5
[0.15.0-preview.4]: https://github.com/VOICEVOX/voicevox_core/compare/0.15.0-preview.3...0.15.0-preview.4
[0.15.0-preview.3]: https://github.com/VOICEVOX/voicevox_core/compare/0.14.0...0.15.0-preview.3

[#370]: https://github.com/VOICEVOX/voicevox_core/pull/370
[#392]: https://github.com/VOICEVOX/voicevox_core/pull/392
[#392]: https://github.com/VOICEVOX/voicevox_core/pull/392
[#400]: https://github.com/VOICEVOX/voicevox_core/pull/400
[#404]: https://github.com/VOICEVOX/voicevox_core/pull/404
[#404]: https://github.com/VOICEVOX/voicevox_core/pull/404
[#407]: https://github.com/VOICEVOX/voicevox_core/pull/407
[#412]: https://github.com/VOICEVOX/voicevox_core/pull/412
[#415]: https://github.com/VOICEVOX/voicevox_core/pull/415
[#415]: https://github.com/VOICEVOX/voicevox_core/pull/415
[#416]: https://github.com/VOICEVOX/voicevox_core/pull/416
[#419]: https://github.com/VOICEVOX/voicevox_core/pull/419
[#420]: https://github.com/VOICEVOX/voicevox_core/pull/420
[#420]: https://github.com/VOICEVOX/voicevox_core/pull/420
[#421]: https://github.com/VOICEVOX/voicevox_core/pull/421
[#421]: https://github.com/VOICEVOX/voicevox_core/pull/421
[#429]: https://github.com/VOICEVOX/voicevox_core/pull/429
[#429]: https://github.com/VOICEVOX/voicevox_core/pull/429
[#432]: https://github.com/VOICEVOX/voicevox_core/pull/432
[#432]: https://github.com/VOICEVOX/voicevox_core/pull/432
[#434]: https://github.com/VOICEVOX/voicevox_core/pull/434
[#438]: https://github.com/VOICEVOX/voicevox_core/pull/438
[#438]: https://github.com/VOICEVOX/voicevox_core/pull/438
[#439]: https://github.com/VOICEVOX/voicevox_core/pull/439
[#439]: https://github.com/VOICEVOX/voicevox_core/pull/439
[#444]: https://github.com/VOICEVOX/voicevox_core/pull/444
[#444]: https://github.com/VOICEVOX/voicevox_core/pull/444
[#450]: https://github.com/VOICEVOX/voicevox_core/pull/450
[#450]: https://github.com/VOICEVOX/voicevox_core/pull/450
[#452]: https://github.com/VOICEVOX/voicevox_core/pull/452
[#452]: https://github.com/VOICEVOX/voicevox_core/pull/452
[#455]: https://github.com/VOICEVOX/voicevox_core/pull/455
[#455]: https://github.com/VOICEVOX/voicevox_core/pull/455
[#458]: https://github.com/VOICEVOX/voicevox_core/pull/458
[#458]: https://github.com/VOICEVOX/voicevox_core/pull/458
[#463]: https://github.com/VOICEVOX/voicevox_core/pull/463
[#463]: https://github.com/VOICEVOX/voicevox_core/pull/463
[#471]: https://github.com/VOICEVOX/voicevox_core/pull/471
[#471]: https://github.com/VOICEVOX/voicevox_core/pull/471
[#473]: https://github.com/VOICEVOX/voicevox_core/pull/473
[#473]: https://github.com/VOICEVOX/voicevox_core/pull/473
[#475]: https://github.com/VOICEVOX/voicevox_core/pull/475
[#475]: https://github.com/VOICEVOX/voicevox_core/pull/475
[#478]: https://github.com/VOICEVOX/voicevox_core/pull/478
[#478]: https://github.com/VOICEVOX/voicevox_core/pull/478
[#479]: https://github.com/VOICEVOX/voicevox_core/pull/479
[#479]: https://github.com/VOICEVOX/voicevox_core/pull/479
[#481]: https://github.com/VOICEVOX/voicevox_core/pull/481
[#481]: https://github.com/VOICEVOX/voicevox_core/pull/481
[#483]: https://github.com/VOICEVOX/voicevox_core/pull/483
[#483]: https://github.com/VOICEVOX/voicevox_core/pull/483
[#485]: https://github.com/VOICEVOX/voicevox_core/pull/485
[#486]: https://github.com/VOICEVOX/voicevox_core/pull/486
[#486]: https://github.com/VOICEVOX/voicevox_core/pull/486
[#487]: https://github.com/VOICEVOX/voicevox_core/pull/487
[#487]: https://github.com/VOICEVOX/voicevox_core/pull/487
[#494]: https://github.com/VOICEVOX/voicevox_core/pull/494
[#495]: https://github.com/VOICEVOX/voicevox_core/pull/495
[#497]: https://github.com/VOICEVOX/voicevox_core/pull/497
[#498]: https://github.com/VOICEVOX/voicevox_core/pull/498
[#500]: https://github.com/VOICEVOX/voicevox_core/pull/500
[#501]: https://github.com/VOICEVOX/voicevox_core/pull/501
[#502]: https://github.com/VOICEVOX/voicevox_core/pull/502
[#503]: https://github.com/VOICEVOX/voicevox_core/pull/503
[#505]: https://github.com/VOICEVOX/voicevox_core/pull/505
[#507]: https://github.com/VOICEVOX/voicevox_core/pull/507
[#511]: https://github.com/VOICEVOX/voicevox_core/pull/511
[#512]: https://github.com/VOICEVOX/voicevox_core/pull/512
[#514]: https://github.com/VOICEVOX/voicevox_core/pull/514
[#516]: https://github.com/VOICEVOX/voicevox_core/pull/516
[#518]: https://github.com/VOICEVOX/voicevox_core/pull/518
[#519]: https://github.com/VOICEVOX/voicevox_core/pull/519
[#520]: https://github.com/VOICEVOX/voicevox_core/pull/520
[#521]: https://github.com/VOICEVOX/voicevox_core/pull/521
[#522]: https://github.com/VOICEVOX/voicevox_core/pull/522
[#523]: https://github.com/VOICEVOX/voicevox_core/pull/523
[#525]: https://github.com/VOICEVOX/voicevox_core/pull/525
[#531]: https://github.com/VOICEVOX/voicevox_core/pull/531
[#532]: https://github.com/VOICEVOX/voicevox_core/pull/532
[#534]: https://github.com/VOICEVOX/voicevox_core/pull/534
[#535]: https://github.com/VOICEVOX/voicevox_core/pull/535
[#536]: https://github.com/VOICEVOX/voicevox_core/pull/536
[#538]: https://github.com/VOICEVOX/voicevox_core/pull/538
[#542]: https://github.com/VOICEVOX/voicevox_core/pull/542
[#546]: https://github.com/VOICEVOX/voicevox_core/pull/546
[#551]: https://github.com/VOICEVOX/voicevox_core/pull/551
[#553]: https://github.com/VOICEVOX/voicevox_core/pull/553
[#555]: https://github.com/VOICEVOX/voicevox_core/pull/555
[#557]: https://github.com/VOICEVOX/voicevox_core/pull/557
[#558]: https://github.com/VOICEVOX/voicevox_core/pull/558
[#565]: https://github.com/VOICEVOX/voicevox_core/pull/565
[#569]: https://github.com/VOICEVOX/voicevox_core/pull/569
[#570]: https://github.com/VOICEVOX/voicevox_core/pull/570
[#571]: https://github.com/VOICEVOX/voicevox_core/pull/571
[#573]: https://github.com/VOICEVOX/voicevox_core/pull/573
[#574]: https://github.com/VOICEVOX/voicevox_core/pull/574
[#575]: https://github.com/VOICEVOX/voicevox_core/pull/575
[#576]: https://github.com/VOICEVOX/voicevox_core/pull/576
[#577]: https://github.com/VOICEVOX/voicevox_core/pull/577
[#579]: https://github.com/VOICEVOX/voicevox_core/pull/579
[#580]: https://github.com/VOICEVOX/voicevox_core/pull/580
[#584]: https://github.com/VOICEVOX/voicevox_core/pull/584
[#586]: https://github.com/VOICEVOX/voicevox_core/pull/586
[#587]: https://github.com/VOICEVOX/voicevox_core/pull/587
[#589]: https://github.com/VOICEVOX/voicevox_core/pull/589
[#590]: https://github.com/VOICEVOX/voicevox_core/pull/590
[#597]: https://github.com/VOICEVOX/voicevox_core/pull/597
[#598]: https://github.com/VOICEVOX/voicevox_core/pull/598
[#600]: https://github.com/VOICEVOX/voicevox_core/pull/600
[#601]: https://github.com/VOICEVOX/voicevox_core/pull/601
[#602]: https://github.com/VOICEVOX/voicevox_core/pull/602
[#603]: https://github.com/VOICEVOX/voicevox_core/pull/603
[#604]: https://github.com/VOICEVOX/voicevox_core/pull/604
[#605]: https://github.com/VOICEVOX/voicevox_core/pull/605
[#611]: https://github.com/VOICEVOX/voicevox_core/pull/611
[#612]: https://github.com/VOICEVOX/voicevox_core/pull/612
[#613]: https://github.com/VOICEVOX/voicevox_core/pull/613
[#616]: https://github.com/VOICEVOX/voicevox_core/pull/616
[#621]: https://github.com/VOICEVOX/voicevox_core/pull/621
[#622]: https://github.com/VOICEVOX/voicevox_core/pull/622
[#623]: https://github.com/VOICEVOX/voicevox_core/pull/623
[#624]: https://github.com/VOICEVOX/voicevox_core/pull/624
[#625]: https://github.com/VOICEVOX/voicevox_core/pull/625
[#626]: https://github.com/VOICEVOX/voicevox_core/pull/626
[#630]: https://github.com/VOICEVOX/voicevox_core/pull/630
[#632]: https://github.com/VOICEVOX/voicevox_core/pull/632
[#640]: https://github.com/VOICEVOX/voicevox_core/pull/640
[#641]: https://github.com/VOICEVOX/voicevox_core/pull/641
[#643]: https://github.com/VOICEVOX/voicevox_core/pull/643
[#646]: https://github.com/VOICEVOX/voicevox_core/pull/646
[#647]: https://github.com/VOICEVOX/voicevox_core/pull/647
[#654]: https://github.com/VOICEVOX/voicevox_core/pull/654
[#656]: https://github.com/VOICEVOX/voicevox_core/pull/656
[#661]: https://github.com/VOICEVOX/voicevox_core/pull/661
[#663]: https://github.com/VOICEVOX/voicevox_core/pull/663
[#664]: https://github.com/VOICEVOX/voicevox_core/pull/664
[#666]: https://github.com/VOICEVOX/voicevox_core/pull/666
[#667]: https://github.com/VOICEVOX/voicevox_core/pull/667
[#668]: https://github.com/VOICEVOX/voicevox_core/pull/668
[#669]: https://github.com/VOICEVOX/voicevox_core/pull/669
[#671]: https://github.com/VOICEVOX/voicevox_core/pull/671
[#673]: https://github.com/VOICEVOX/voicevox_core/pull/673
[#675]: https://github.com/VOICEVOX/voicevox_core/pull/675
[#678]: https://github.com/VOICEVOX/voicevox_core/pull/678
[#682]: https://github.com/VOICEVOX/voicevox_core/pull/682
[#684]: https://github.com/VOICEVOX/voicevox_core/pull/684
[#692]: https://github.com/VOICEVOX/voicevox_core/pull/692
[#694]: https://github.com/VOICEVOX/voicevox_core/pull/694
[#695]: https://github.com/VOICEVOX/voicevox_core/pull/695
[#696]: https://github.com/VOICEVOX/voicevox_core/pull/696
[#699]: https://github.com/VOICEVOX/voicevox_core/pull/699
[#702]: https://github.com/VOICEVOX/voicevox_core/pull/702
[#706]: https://github.com/VOICEVOX/voicevox_core/pull/706
[#707]: https://github.com/VOICEVOX/voicevox_core/pull/707
[#708]: https://github.com/VOICEVOX/voicevox_core/pull/708
[#719]: https://github.com/VOICEVOX/voicevox_core/pull/719
[#720]: https://github.com/VOICEVOX/voicevox_core/pull/720
[#723]: https://github.com/VOICEVOX/voicevox_core/pull/723
[#724]: https://github.com/VOICEVOX/voicevox_core/pull/724
[#725]: https://github.com/VOICEVOX/voicevox_core/pull/725
[#728]: https://github.com/VOICEVOX/voicevox_core/pull/728
[#733]: https://github.com/VOICEVOX/voicevox_core/pull/733
[#738]: https://github.com/VOICEVOX/voicevox_core/pull/738
[#740]: https://github.com/VOICEVOX/voicevox_core/pull/740
[#745]: https://github.com/VOICEVOX/voicevox_core/pull/745
[#747]: https://github.com/VOICEVOX/voicevox_core/pull/747
[#752]: https://github.com/VOICEVOX/voicevox_core/pull/752
[#753]: https://github.com/VOICEVOX/voicevox_core/pull/753
[#759]: https://github.com/VOICEVOX/voicevox_core/pull/759
[#761]: https://github.com/VOICEVOX/voicevox_core/pull/761
[#764]: https://github.com/VOICEVOX/voicevox_core/pull/764
[#794]: https://github.com/VOICEVOX/voicevox_core/pull/794
[#795]: https://github.com/VOICEVOX/voicevox_core/pull/795
[#796]: https://github.com/VOICEVOX/voicevox_core/pull/796
[#800]: https://github.com/VOICEVOX/voicevox_core/pull/800
[#801]: https://github.com/VOICEVOX/voicevox_core/pull/801
[#802]: https://github.com/VOICEVOX/voicevox_core/pull/802
[#803]: https://github.com/VOICEVOX/voicevox_core/pull/803
[#805]: https://github.com/VOICEVOX/voicevox_core/pull/805
[#806]: https://github.com/VOICEVOX/voicevox_core/pull/806
[#807]: https://github.com/VOICEVOX/voicevox_core/pull/807
[#810]: https://github.com/VOICEVOX/voicevox_core/pull/810
[#822]: https://github.com/VOICEVOX/voicevox_core/pull/822
[#823]: https://github.com/VOICEVOX/voicevox_core/pull/823
[#824]: https://github.com/VOICEVOX/voicevox_core/pull/824
[#825]: https://github.com/VOICEVOX/voicevox_core/pull/825
[#830]: https://github.com/VOICEVOX/voicevox_core/pull/830
[#831]: https://github.com/VOICEVOX/voicevox_core/pull/831
[#832]: https://github.com/VOICEVOX/voicevox_core/pull/832
[#834]: https://github.com/VOICEVOX/voicevox_core/pull/834
[#835]: https://github.com/VOICEVOX/voicevox_core/pull/835
[#837]: https://github.com/VOICEVOX/voicevox_core/pull/837
[#838]: https://github.com/VOICEVOX/voicevox_core/pull/838
[#844]: https://github.com/VOICEVOX/voicevox_core/pull/844
[#846]: https://github.com/VOICEVOX/voicevox_core/pull/846
[#847]: https://github.com/VOICEVOX/voicevox_core/pull/847
[#849]: https://github.com/VOICEVOX/voicevox_core/pull/849
[#850]: https://github.com/VOICEVOX/voicevox_core/pull/850
[#856]: https://github.com/VOICEVOX/voicevox_core/pull/856
[#860]: https://github.com/VOICEVOX/voicevox_core/pull/860
[#861]: https://github.com/VOICEVOX/voicevox_core/pull/861
[#862]: https://github.com/VOICEVOX/voicevox_core/pull/862
[#863]: https://github.com/VOICEVOX/voicevox_core/pull/863
[#868]: https://github.com/VOICEVOX/voicevox_core/pull/868
[#876]: https://github.com/VOICEVOX/voicevox_core/pull/876
[#881]: https://github.com/VOICEVOX/voicevox_core/pull/881
[#882]: https://github.com/VOICEVOX/voicevox_core/pull/882
[#884]: https://github.com/VOICEVOX/voicevox_core/pull/884
[#886]: https://github.com/VOICEVOX/voicevox_core/pull/886
[#887]: https://github.com/VOICEVOX/voicevox_core/pull/887
[#889]: https://github.com/VOICEVOX/voicevox_core/pull/889
[#890]: https://github.com/VOICEVOX/voicevox_core/pull/890
[#895]: https://github.com/VOICEVOX/voicevox_core/pull/895
[#898]: https://github.com/VOICEVOX/voicevox_core/pull/898
[#903]: https://github.com/VOICEVOX/voicevox_core/pull/903
[#907]: https://github.com/VOICEVOX/voicevox_core/pull/907
[#910]: https://github.com/VOICEVOX/voicevox_core/pull/910
[#911]: https://github.com/VOICEVOX/voicevox_core/pull/911
[#912]: https://github.com/VOICEVOX/voicevox_core/pull/912
[#913]: https://github.com/VOICEVOX/voicevox_core/pull/913
[#914]: https://github.com/VOICEVOX/voicevox_core/pull/914
[#915]: https://github.com/VOICEVOX/voicevox_core/pull/915
[#919]: https://github.com/VOICEVOX/voicevox_core/pull/919
[#921]: https://github.com/VOICEVOX/voicevox_core/pull/921
[#926]: https://github.com/VOICEVOX/voicevox_core/pull/926
[#927]: https://github.com/VOICEVOX/voicevox_core/pull/927
[#928]: https://github.com/VOICEVOX/voicevox_core/pull/928
[#930]: https://github.com/VOICEVOX/voicevox_core/pull/930
[#931]: https://github.com/VOICEVOX/voicevox_core/pull/931
[#932]: https://github.com/VOICEVOX/voicevox_core/pull/932
[#933]: https://github.com/VOICEVOX/voicevox_core/pull/933
[#935]: https://github.com/VOICEVOX/voicevox_core/pull/935
[#937]: https://github.com/VOICEVOX/voicevox_core/pull/937
[#939]: https://github.com/VOICEVOX/voicevox_core/pull/939
[#940]: https://github.com/VOICEVOX/voicevox_core/pull/940
[#941]: https://github.com/VOICEVOX/voicevox_core/pull/941
[#942]: https://github.com/VOICEVOX/voicevox_core/pull/942
[#943]: https://github.com/VOICEVOX/voicevox_core/pull/943
[#944]: https://github.com/VOICEVOX/voicevox_core/pull/944
[#945]: https://github.com/VOICEVOX/voicevox_core/pull/945
[#946]: https://github.com/VOICEVOX/voicevox_core/pull/946
[#947]: https://github.com/VOICEVOX/voicevox_core/pull/947
[#949]: https://github.com/VOICEVOX/voicevox_core/pull/949
[#950]: https://github.com/VOICEVOX/voicevox_core/pull/950
[#952]: https://github.com/VOICEVOX/voicevox_core/pull/952
[#953]: https://github.com/VOICEVOX/voicevox_core/pull/953
[#954]: https://github.com/VOICEVOX/voicevox_core/pull/954
[#955]: https://github.com/VOICEVOX/voicevox_core/pull/955
[#957]: https://github.com/VOICEVOX/voicevox_core/pull/957
[#958]: https://github.com/VOICEVOX/voicevox_core/pull/958
[#959]: https://github.com/VOICEVOX/voicevox_core/pull/959
[#964]: https://github.com/VOICEVOX/voicevox_core/pull/964
[#965]: https://github.com/VOICEVOX/voicevox_core/pull/965
[#967]: https://github.com/VOICEVOX/voicevox_core/pull/967
[#969]: https://github.com/VOICEVOX/voicevox_core/pull/969
[#973]: https://github.com/VOICEVOX/voicevox_core/pull/973
[#974]: https://github.com/VOICEVOX/voicevox_core/pull/974
[#976]: https://github.com/VOICEVOX/voicevox_core/pull/976
[#977]: https://github.com/VOICEVOX/voicevox_core/pull/977
[#979]: https://github.com/VOICEVOX/voicevox_core/pull/979
[#980]: https://github.com/VOICEVOX/voicevox_core/pull/980
[#982]: https://github.com/VOICEVOX/voicevox_core/pull/982
[#983]: https://github.com/VOICEVOX/voicevox_core/pull/983
[#984]: https://github.com/VOICEVOX/voicevox_core/issues/984
[#985]: https://github.com/VOICEVOX/voicevox_core/pull/985
[#986]: https://github.com/VOICEVOX/voicevox_core/pull/986
[#988]: https://github.com/VOICEVOX/voicevox_core/pull/988
[#989]: https://github.com/VOICEVOX/voicevox_core/pull/989
[#990]: https://github.com/VOICEVOX/voicevox_core/pull/990
[#991]: https://github.com/VOICEVOX/voicevox_core/pull/991
[#992]: https://github.com/VOICEVOX/voicevox_core/pull/992
[#993]: https://github.com/VOICEVOX/voicevox_core/pull/993
[#994]: https://github.com/VOICEVOX/voicevox_core/pull/994
[#995]: https://github.com/VOICEVOX/voicevox_core/pull/995
[#996]: https://github.com/VOICEVOX/voicevox_core/pull/996
[#997]: https://github.com/VOICEVOX/voicevox_core/pull/997
[#998]: https://github.com/VOICEVOX/voicevox_core/pull/998
[#999]: https://github.com/VOICEVOX/voicevox_core/pull/999
[#1002]: https://github.com/VOICEVOX/voicevox_core/pull/1002
[#1003]: https://github.com/VOICEVOX/voicevox_core/pull/1003
[#1006]: https://github.com/VOICEVOX/voicevox_core/pull/1006
[#1011]: https://github.com/VOICEVOX/voicevox_core/pull/1011
[#1014]: https://github.com/VOICEVOX/voicevox_core/pull/1014
[#1015]: https://github.com/VOICEVOX/voicevox_core/pull/1015
[#1016]: https://github.com/VOICEVOX/voicevox_core/pull/1016
[#1019]: https://github.com/VOICEVOX/voicevox_core/pull/1019
[#1020]: https://github.com/VOICEVOX/voicevox_core/pull/1020
[#1021]: https://github.com/VOICEVOX/voicevox_core/pull/1021
[#1023]: https://github.com/VOICEVOX/voicevox_core/pull/1023
[#1024]: https://github.com/VOICEVOX/voicevox_core/pull/1024
[#1025]: https://github.com/VOICEVOX/voicevox_core/pull/1025
[#1028]: https://github.com/VOICEVOX/voicevox_core/pull/1028
[#1030]: https://github.com/VOICEVOX/voicevox_core/pull/1030
[#1032]: https://github.com/VOICEVOX/voicevox_core/pull/1032
[#1033]: https://github.com/VOICEVOX/voicevox_core/pull/1033
[#1034]: https://github.com/VOICEVOX/voicevox_core/pull/1034
[#1035]: https://github.com/VOICEVOX/voicevox_core/pull/1035
[#1040]: https://github.com/VOICEVOX/voicevox_core/pull/1040
[#1041]: https://github.com/VOICEVOX/voicevox_core/pull/1041
[#1043]: https://github.com/VOICEVOX/voicevox_core/pull/1043
[#1044]: https://github.com/VOICEVOX/voicevox_core/pull/1044
[#1045]: https://github.com/VOICEVOX/voicevox_core/pull/1045
[#1048]: https://github.com/VOICEVOX/voicevox_core/pull/1048
[#1049]: https://github.com/VOICEVOX/voicevox_core/pull/1049
[#1055]: https://github.com/VOICEVOX/voicevox_core/pull/1055
[#1056]: https://github.com/VOICEVOX/voicevox_core/pull/1056
[#1057]: https://github.com/VOICEVOX/voicevox_core/pull/1057
[#1058]: https://github.com/VOICEVOX/voicevox_core/pull/1058
[#1060]: https://github.com/VOICEVOX/voicevox_core/pull/1060
[#1063]: https://github.com/VOICEVOX/voicevox_core/pull/1063
[#1073]: https://github.com/VOICEVOX/voicevox_core/pull/1073
[#1077]: https://github.com/VOICEVOX/voicevox_core/pull/1077
[#1078]: https://github.com/VOICEVOX/voicevox_core/pull/1078
[#1082]: https://github.com/VOICEVOX/voicevox_core/pull/1082
[#1085]: https://github.com/VOICEVOX/voicevox_core/pull/1085
[#1094]: https://github.com/VOICEVOX/voicevox_core/pull/1094
[#1096]: https://github.com/VOICEVOX/voicevox_core/pull/1096
[#1098]: https://github.com/VOICEVOX/voicevox_core/pull/1098
[#1099]: https://github.com/VOICEVOX/voicevox_core/pull/1099
[#1100]: https://github.com/VOICEVOX/voicevox_core/pull/1100
[#1103]: https://github.com/VOICEVOX/voicevox_core/issues/1103
[#1108]: https://github.com/VOICEVOX/voicevox_core/pull/1108
[#1111]: https://github.com/VOICEVOX/voicevox_core/pull/1111

[VOICEVOX/onnxruntime-builder#25]: https://github.com/VOICEVOX/onnxruntime-builder/pull/25

[VOICEVOX/voicevox\_vvm#12]: https://github.com/VOICEVOX/voicevox_vvm/pull/12
[VOICEVOX/voicevox\_vvm#14]: https://github.com/VOICEVOX/voicevox_vvm/pull/14

[dtolnay/proc-macro2#497]: https://github.com/dtolnay/proc-macro2/pull/497

[@char5742]: https://github.com/char5742
[@cm-ayf]: https://github.com/cm-ayf
[@eyr1n]: https://github.com/eyr1n
[@fuziki]: https://github.com/fuziki
[@HyodaKazuaki]: https://github.com/HyodaKazuaki
[@nanae772]: https://github.com/nanae772
[@nekomimimi]: https://github.com/nekomimimi
[@phenylshima]: https://github.com/phenylshima
[@sh1ma]: https://github.com/sh1ma
[@shigobu]: https://github.com/shigobu
[@shuntia]: https://github.com/shuntia
[@weweweok]: https://github.com/weweweok
