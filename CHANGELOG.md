# Changelog

## [Unreleased]

TODO: 執筆中

## [0.16.0] - 2025-03-29 (+09:00)

TODO: 執筆中

## [0.16.0-preview.1] - 2025-03-08 (+09:00)

TODO: 執筆中

## [0.16.0-preview.0] - 2025-03-01 (+09:00)

TODO: 執筆中

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

    `--only <TARGET>...`で限定、`--exclude <TARGET>...`で除外ができます。

- \[Python,Java\] エラーの文脈が例外チェーンとしてくっつくようになりました ([#640])。

### Changed

- \[Python,Java\] `VoicevoxError`/`VoicevoxException`は解体され、個別のエラーになります ([#640])。

### Fixed

- \[Java\] JVMのMUTF-8 (modified UTF-8)の文字列が誤ってUTF-8として読まれていた問題が解決されます ([#654])。

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

[@char5742]: https://github.com/char5742
[@fuziki]: https://github.com/fuziki
[@HyodaKazuaki]: https://github.com/HyodaKazuaki
[@sh1ma]: https://github.com/sh1ma
[@shigobu]: https://github.com/shigobu
[@weweweok]: https://github.com/weweweok
