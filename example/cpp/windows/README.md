# Windows C++ のサンプルプロジェクト

ここには、voicevox_coreライブラリをC++から使用するサンプルプロジェクトが含まれています。
プロジェクトを開くには、Microsoft Visual Studio Community 2022(無料)が必要です。また、「C++によるデスクトップ開発」のワークロードが必要です。
Visual Studio Installerを使用しインストールしてください。  

## simple_tts

単純な音声合成を行うコンソールアプリケーションです。

### 環境構築・ビルド方法

以下はDebug x64でビルドする場合です。他の構成・プラットフォーム向けにビルドする場合は、適宜読み替えてください。  

出力フォルダを作成するために、一度ビルドします。「windows_example.sln」をVisual Studioで開き、メニューの「ビルド」→「ソリューションのビルド」を押します。
この段階では、ビルドは失敗します。「bin」フォルダと「lib」フォルダが生成されていればOKです。  
[Releases](https://github.com/VOICEVOX/voicevox_core/releases/latest)から「voicevox_core-windows-x64-{バージョン名}.zip」をダウンロードし、展開します。[ダウンローダー](https://github.com/VOICEVOX/voicevox_core/blob/main/docs/guide/user/downloader.md)を使うと便利です。  
展開してできたファイル・フォルダをそれぞれ下記のフォルダへ配置します。

- simple_tts に配置
  - voicevox_core.h

- simple_tts\bin\x64\Debug に配置
  - voicevox_core.dll
  - voicevox_onnxruntime.dll
  - modelsフォルダ

- simple_tts\lib\x64 に配置
  - voicevox_core.lib

もう一度ビルドします。今度は成功するはずです。失敗した場合は、「voicevox_core.lib」の場所を確認してください。

続いて、「Open JTalk辞書フォルダ」を配置します。  
ダウンローダーを使用した場合、展開されたフォルダに含まれています。  
ダウンローダーを使用していない場合は、http://open-jtalk.sourceforge.net/ を開き、Dictionary for Open JTalk 欄の Binary Package (UTF-8)をクリックして「open_jtalk_dic_utf_8-1.11.tar.gz」をダウンロードします。  

展開してできた「open_jtalk_dic_utf_8-1.11」フォルダをフォルダごと「simple_tts\bin\x64\Debug」に配置します。

最終的に、以下のようなフォルダ構成になっているはずです。
```
simple_tts
 │  simple_tts.cpp
 │  simple_tts.h
 │  simple_tts.vcxproj
 │  simple_tts.vcxproj.filters
 │  simple_tts.vcxproj.user
 │  voicevox_core.h
 │
 ├─bin
 │  └─x64
 │      └─Debug
 │         │  simple_tts.exe
 │         │  simple_tts.pdb
 │         │  voicevox_core.dll
 │         │  voicevox_onnxruntime.dll
 │         │
 │         ├─models
 │         │     vvms
 │         │
 │         └─open_jtalk_dic_utf_8-1.11
 │
 └─lib
     └─x64
         │  voicevox_core.lib
         │
         └─Debug
```

## song

歌唱音声合成を行うサンプルコードはまだ用意されていません。代わりに[Linux・macOS向けのsong.cpp](../unix/song.cpp)を参考にしてください。

### 実行
Visual Studioのツールバーにある「ローカル Windows デバッガー」と書いてある三角のついているボタンを押すと実行できます。出力フォルダにある「simple_tts.exe」を直接実行することもできます。
表示されたコンソール画面に、生成したい音声の文字列を入力しエンターキーを押します。そうすると音声合成が開始し、合成された音声が再生されます。
