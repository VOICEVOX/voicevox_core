# VOICEVOX CORE

> [!NOTE]
> プレビュー版ビルドの`0.15.0-preview.*`および`0.16.0-preview.*`は削除予定です。\
> 早めの最新バージョン`0.16`への移行を推奨します。詳しくは[こちら](https://github.com/VOICEVOX/voicevox_core/issues/1067)。

[![releases](https://img.shields.io/github/v/release/VOICEVOX/voicevox_core?label=release)](https://github.com/VOICEVOX/voicevox_core/releases)
[![test](https://github.com/VOICEVOX/voicevox_core/actions/workflows/test.yml/badge.svg)](https://github.com/VOICEVOX/voicevox_core/actions/workflows/test.yml)
[![dependency status](https://deps.rs/repo/github/VOICEVOX/voicevox_core/status.svg)](https://deps.rs/repo/github/VOICEVOX/voicevox_core)
[![discord](https://img.shields.io/discord/879570910208733277?color=5865f2&label=&logo=discord&logoColor=ffffff)](https://discord.gg/WMwWetrzuh)

[VOICEVOX](https://voicevox.hiroshiba.jp/) の音声合成コア。  
[Releases](https://github.com/VOICEVOX/voicevox_core/releases) に以下のビルド済みのライブラリがあります。

- C APIの動的ライブラリ（.so/.dll/.dylib）
- Python APIのwheel（.whl）

（エディターは [VOICEVOX](https://github.com/VOICEVOX/voicevox/) 、
エンジンは [VOICEVOX ENGINE](https://github.com/VOICEVOX/voicevox_engine/) 、
全体構成は [こちら](https://github.com/VOICEVOX/voicevox/blob/main/docs/%E5%85%A8%E4%BD%93%E6%A7%8B%E6%88%90.md) に詳細があります。）

## API

[API ドキュメント](https://voicevox.github.io/voicevox_core/apis/)をご覧ください。

## ユーザーガイド

[VOICEVOX コア ユーザーガイド](./docs/guide/user/usage.md)をご覧ください。

## サンプル実行

現在このリポジトリでは次のサンプルが提供されています。実行方法についてはそれぞれのディレクトリ内にある README を参照してください

- [Python(pip)](./example/python)
- [C++(UNIX CMake)](./example/cpp/unix)
- [C++(Windows Visual Studio)](./example/cpp/windows)

### その他の言語

- [Go(Windows)](https://github.com/yerrowTail/voicevox_core_go_sample) @yerrowTail
- [C#](https://github.com/yamachu/VoicevoxCoreSharp) @yamachu

サンプルコードを実装された際はぜひお知らせください。こちらに追記させて頂きます。

## 事例紹介

**[voicevox.rb](https://github.com/sevenc-nanashi/voicevox.rb) [@sevenc-nanashi](https://github.com/sevenc-nanashi)** ･･･ VOICEVOX CORE の Ruby 向け FFI ラッパー  
**[Node VOICEVOX Engine](https://github.com/y-chan/node-voicevox-engine) [@y-chan](https://github.com/y-chan)** ･･･ VOICEVOX ENGINE の Node.js/C++ 実装  
**[VOICEVOX ENGINE SHARP](https://github.com/yamachu/VoicevoxEngineSharp) [@yamachu](https://github.com/yamachu)** ･･･ VOICEVOX ENGINE の C# 実装  
**[voicevoxcore4s](https://github.com/windymelt/voicevoxcore4s) [@windymelt](https://github.com/windymelt)** ･･･ VOICEVOX CORE の Scala(JVM) 向け FFI ラッパー  
**[voicevox_flutter](https://github.com/char5742/voicevox_flutter) [@char5742](https://github.com/char5742)** ･･･ VOICEVOX CORE の Flutter 向け FFI ラッパー  
**[voicevoxcore.go](https://github.com/sh1ma/voicevoxcore.go) [@sh1ma](https://github.com/sh1ma)** ･･･ VOICEVOX CORE の Go 言語 向け FFI ラッパー  
**[VoicevoxCoreSharp](https://github.com/yamachu/VoicevoxCoreSharp) [@yamachu](https://github.com/yamachu)** ･･･ VOICEVOX CORE の C# 向け FFI ラッパー  
**[VoicevoxCoreSwift](https://github.com/yamachu/VoicevoxCoreSwift) [@yamachu](https://github.com/yamachu)** ･･･ VOICEVOX CORE の Swift 向け FFI ラッパー  

## ライセンス

VOICEVOX CORE のソースコード及びビルド成果物のライセンスは [MIT LICENSE](./LICENSE) です。

[Releases](https://github.com/VOICEVOX/voicevox_core/releases) にあるバージョン 0.16 未満のビルド済みのコアライブラリは別ライセンスなのでご注意ください。
