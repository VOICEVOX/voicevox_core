# Windows C++ のサンプルプロジェクト

ここには、voicevox_coreライブラリをC++から使用するサンプルプロジェクトが含まれています。
プロジェクトを開くには、Microsoft Visual Studio Community 2022(無料)が必要です。また、「C++によるデスクトップ開発」のワークロードが必要です。
Visual Studio Installerを使用しインストールしてください。  

## simple_tts

単純な音声合成を行うコンシールアプリケーションです。

### 環境構築・ビルド方法

ビルドして実行するには、「core.dll」「core.lib」「Open JTalk辞書フォルダ」が必要です。
以下はDebug x64でビルドする場合です。他の構成・プラットフォーム向けにビルドする場合は、適宜読み替えてください。  

Releasesから「voicevox_core-windows-x64-cpu-{バージョン名}.zip」をダウンロードします。
zipファイルを展開し、展開されたフォルダに含まれているdllファイルを「core.dll」にリネームします。
出力フォルダを作成するために、一度ビルドします。「windows_example.sln」をVisual Studioで開き、メニューの「ビルド」→「ソリューションのビルド」を押します。
この段階では、ビルドは失敗します。「bin」フォルダと「lib」フォルダが生成されていればOKです。  

「core.lib」を「simple_tts\lib\x64」に配置します。  
「core.dll」を「simple_tts\bin\x64\Debug」に配置します。

もう一度ビルドします。今度は成功するはずです。失敗した場合は、「core.lib」の場所を確認してください。

続いて、「Open JTalk辞書フォルダ」を配置します。
http://open-jtalk.sourceforge.net/ を開き、Dictionary for Open JTalk 欄の Binary Package (UTF-8)をクリックして「open_jtalk_dic_utf_8-1.11.tar.gz」をダウンロードします。  

展開してできた「open_jtalk_dic_utf_8-1.11」フォルダをフォルダごと「simple_tts\bin\x64\Debug」に配置します。

最終的には以下のようなフォルダ構成になっているはずです。
```
simple_tts
│  packages.config
│  simple_tts.cpp
│  simple_tts.h
│  simple_tts.vcxproj
│  simple_tts.vcxproj.filters
│  simple_tts.vcxproj.user
│
├─bin
│  └─x64
│      └─Debug
│          │  core.dll
│          │  onnxruntime.dll
│          │  onnxruntime_providers_shared.dll
│          │  simple_tts.exe
│          │  simple_tts.pdb
│          │
│          └─open_jtalk_dic_utf_8-1.11
│
└─lib
    └─x64
        │  core.lib
        │
        └─Debug
```

### 実行
Visual Studioのツールバーにある「ローカル Windows デバッガー」と書いてある三角のつているボタンを押すと実行できます。出力フォルダにある「simple_tts.exe」を直接実行することもできます。
表示されたコンソール画面に、生成したい音声の文字列を入力しエンターキーを押します。そうすると音声合成が開始し、合成された音声が再生されます。
