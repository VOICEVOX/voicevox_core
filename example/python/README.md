# python のサンプルコード

python から voicevox_core ライブラリを使用するためのサンプルコードです。

## サンプル実行方法

まず、この README があるディレクトリで、[downloader を使用して voicevox_core をダウンロードします](../../docs/downloads/Download.md#一般的な実行方法)。  
次に下記コマンドを実行して python のサンプルを実行します。

```bash
# サンプルコード実行のための依存モジュールのインストール
pip install -r requirements.txt
python run.py \
    --text "これは本当に実行できているんですか" \
    --speaker_id 1

# 引数の紹介
# --text 読み上げるテキスト
# --speaker_id 話者ID
# --use_gpu GPUを使う
# --f0_speaker_id 音高の話者ID（デフォルト値はspeaker_id）
# --f0_correct 音高の補正値（デフォルト値は0。+-0.3くらいで結果が大きく変わります）
```
