# VOICEVOX CORE

[VOICEVOX](https://voicevox.hiroshiba.jp/) の音声合成コア。

[Releases](https://github.com/Hiroshiba/voicevox_core/releases) にビルド済みのコアライブラリ（.so/.dll）があります。

## 依存関係

[CUDA 11.1](https://developer.nvidia.com/cuda-11.1.0-download-archive) と [CUDNN](https://developer.nvidia.com/cudnn) のインストールと [LibTorch](https://pytorch.org/) のダウンロードが必要です。

## サンプルの実行

まず [Releases](https://github.com/Hiroshiba/voicevox_core/releases) からコアライブラリが入った zip をダウンロードしておきます。

### Python 3

```bash
cd example/python

# example/python のディレクトリにコアライブラリが入った zip ファイルを展開

# Windowsの場合、DLLからLIBファイルの作成
./makelib.bat core

# 環境構築
pip install -r requirements.txt
python setup.py install

# # うまく行かないときは毎回以下を実行すると良いかも
# python setup.py clean
# rm -r build *.cpp

# 実行（Windowsの場合）
PATH="$PATH:$HOME/libtorch/lib/" python run.py \
    --text "これは本当に実行できているんですか" \
    --speaker_id 1

# 実行（Windows以外の場合）
LD_LIBRARY_PATH="$LD_LIBRARY_PATH:/libtorch/lib/" python run.py \
    --text "これは本当に実行できているんですか" \
    --speaker_id 1

# 引数の紹介
# --text 読み上げるテキスト
# --speaker_id 話者ID
# --use_gpu GPUを使う
# --f0_speaker_id 音高の話者ID（デフォルト値はspeaker_id）
# --f0_correct 音高の補正値（デフォルト値は0。+-0.3くらいで結果が大きく変わります）
```

「ImportError: DLL load failed: 指定されたモジュールが見つかりません。」というエラーが出た場合は libtorch のパスが間違っているかもしれません。

### C#

**[VOICEVOX ENGINE SHARP](https://github.com/yamachu/VoicevoxEngineSharp) @yamachu**  
VOICEVOX ENGINE の C# 実装。

### その他の言語

サンプルコードを実装された際はぜひお知らせください。ここに追記させて頂きます。

## ライセンス

サンプルコードおよび[core.h](./core.h)は [MIT LICENSE](./LICENSE) です。

[Releases](https://github.com/Hiroshiba/voicevox_core/releases) にあるビルド済みのコアライブラリは別ライセンスなのでご注意ください。
