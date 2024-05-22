# VVM ファイル

***VVM ファイル*** は、音声合成に必要な声情報を含むファイルである。  

より正確には、音声合成のモデル重みファイルなどを含む zip 形式のアーカイブである。拡張子は `.vvm`。  
以下の内部ディレクトリ構造を持つ：  

- `{filename}.vvm`
  - `manifest.json`
  - `metas.json`
  - <duration_model>
  - <intonation_model>
  - <decode_model>

model は `.onnx` や `.bin` など様々ある。例えば `sample.vvm` は `predict_duration.onnx` / `predict_intonation.onnx` / `decode.onnx` を含む。  

VOICEVOX OSS が提供する VVM には `sample.vvm` がある（ビルドを行うと `crates/test_util/data/model/sample.vvm` が生成される）。  
製品版 VOICEVOX で利用される VVM は [こちらのレポジトリ](https://github.com/VOICEVOX/voicevox_fat_resource/tree/main/core/model) で確認できる。  

## マニフェストファイル

VVM における ***マニフェストファイル*** は、VVM ファイルの構成や、onnx モデルなどを読み込む・利用するのに必要な情報を記述したファイルである。  
json 形式で記述され、root パスに`manifest.json`として配置する。  
[VOICEVOX CORE のソースコード](https://github.com/VOICEVOX/voicevox_core/blob/main/crates/voicevox_core/src/manifest.rs) 内で `Manifest` 構造体としてスキーマが定義されている。  
