# Python サンプルコード (PyO3 によるバインディング経由)

voicevox_core ライブラリ の Python バインディングを使った音声合成のサンプルコードです。  
`pip install`で導入することができます。

## 準備

TODO

- Python インタプリタ ≧3.8 + venv
- voicevox_core_python_api の whl (`pip install`)
- onnxruntime の DLL (/README.md と同様)
- open_jtalk_dic_utf_8-1.11 (/README.md と同様)

## 実行

VVMファイル、Open JTalk 辞書ディレクトリ、読み上げさせたい文章、出力 wav ファイルのパスの 4 つを指定して run.py を実行します。

```console
❯ python ./run.py -h
usage: run.py [-h] [--mode MODE] vvm open_jtalk_dict_dir text out

positional arguments:
  vvm                  vvmファイルへのパス
  open_jtalk_dict_dir  Open JTalkの辞書ディレクトリ
  text                 読み上げさせたい文章
  out                  出力wavファイルのパス

optional arguments:
  -h, --help           show this help message and exit
  --mode MODE          モード ("AUTO", "CPU", "GPU")
```

```console
❯ # python ./run.py <VVMファイル> <Open JTalk辞書ディレクトリ> <読み上げさせたい文章> <出力wavファイルのパス>
❯ python ./run.py ../../model/sample.vvm ./open_jtalk_dic_utf_8-1.11 これはテストです ./audio.wav
[DEBUG] run.py: voicevox_core.supported_devices()=SupportedDevices(cpu=True, cuda=False, dml=False)
[INFO] run.py: Initializing (acceleration_mode=<AccelerationMode.AUTO: 'AUTO'>, open_jtalk_dict_dir=PosixPath('open_jtalk_dic_utf_8-1.11'))
[DEBUG] run.py: synthesizer.metas=[]
[DEBUG] run.py: synthesizer.is_gpu_mode=False
[INFO] run.py: Loading `../../model/sample.vvm`
[DEBUG] mod.rs: Locate EOCDR
[DEBUG] mod.rs: EOCDR: EndOfCentralDirectoryHeader { disk_num: 0, start_cent_dir_disk: 0, num_of_entries_disk: 5, num_of_entries: 5, size_cent_dir: 308, cent_dir_offset: 53489411, file_comm_length: 0 }
[DEBUG] mod.rs: Zip64EOCDR: None
[DEBUG] mod.rs: Combined directory: CombinedCentralDirectoryRecord { version_made_by: None, version_needed_to_extract: None, disk_number: 0, disk_number_start_of_cd: 0, num_entries_in_directory_on_disk: 5, num_entries_in_directory: 5, directory_size: 308, offset_of_start_of_directory: 53489411, file_comment_length: 0 }
[DEBUG] mod.rs: Read central directory
[DEBUG] mod.rs: Entry: ZipEntry { filename: "metas.json", compression: Deflate, compression_level: Default, crc32: 1124046543, uncompressed_size: 651, compressed_size: 218, attribute_compatibility: Unix, last_modification_date: ZipDateTime { date: 0, time: 0 }, internal_file_attribute: 0, external_file_attribute: 0, extra_fields: [], comment: "" }, offset 0
[DEBUG] mod.rs: Entry: ZipEntry { filename: "predict_intonation.onnx", compression: Deflate, compression_level: Default, crc32: 2603102279, uncompressed_size: 30803, compressed_size: 25591, attribute_compatibility: Unix, last_modification_date: ZipDateTime { date: 0, time: 0 }, internal_file_attribute: 0, external_file_attribute: 0, extra_fields: [], comment: "" }, offset 274
[DEBUG] mod.rs: Entry: ZipEntry { filename: "decode.onnx", compression: Deflate, compression_level: Default, crc32: 1207829611, uncompressed_size: 57149888, compressed_size: 53414353, attribute_compatibility: Unix, last_modification_date: ZipDateTime { date: 0, time: 0 }, internal_file_attribute: 0, external_file_attribute: 0, extra_fields: [], comment: "" }, offset 25934
[DEBUG] mod.rs: Entry: ZipEntry { filename: "predict_duration.onnx", compression: Deflate, compression_level: Default, crc32: 4236608546, uncompressed_size: 53381, compressed_size: 48835, attribute_compatibility: Unix, last_modification_date: ZipDateTime { date: 0, time: 0 }, internal_file_attribute: 0, external_file_attribute: 0, extra_fields: [], comment: "" }, offset 53440344
[DEBUG] mod.rs: Entry: ZipEntry { filename: "manifest.json", compression: Deflate, compression_level: Default, crc32: 64471537, uncompressed_size: 220, compressed_size: 106, attribute_compatibility: Unix, last_modification_date: ZipDateTime { date: 0, time: 0 }, internal_file_attribute: 0, external_file_attribute: 0, extra_fields: [], comment: "" }, offset 53489246
[DEBUG] mod.rs: Locate EOCDR
[DEBUG] mod.rs: EOCDR: EndOfCentralDirectoryHeader { disk_num: 0, start_cent_dir_disk: 0, num_of_entries_disk: 5, num_of_entries: 5, size_cent_dir: 308, cent_dir_offset: 53489411, file_comm_length: 0 }
[DEBUG] mod.rs: Zip64EOCDR: None
[DEBUG] mod.rs: Combined directory: CombinedCentralDirectoryRecord { version_made_by: None, version_needed_to_extract: None, disk_number: 0, disk_number_start_of_cd: 0, num_entries_in_directory_on_disk: 5, num_entries_in_directory: 5, directory_size: 308, offset_of_start_of_directory: 53489411, file_comment_length: 0 }
[DEBUG] mod.rs: Read central directory
[DEBUG] mod.rs: Entry: ZipEntry { filename: "metas.json", compression: Deflate, compression_level: Default, crc32: 1124046543, uncompressed_size: 651, compressed_size: 218, attribute_compatibility: Unix, last_modification_date: ZipDateTime { date: 0, time: 0 }, internal_file_attribute: 0, external_file_attribute: 0, extra_fields: [], comment: "" }, offset 0
[DEBUG] mod.rs: Entry: ZipEntry { filename: "predict_intonation.onnx", compression: Deflate, compression_level: Default, crc32: 2603102279, uncompressed_size: 30803, compressed_size: 25591, attribute_compatibility: Unix, last_modification_date: ZipDateTime { date: 0, time: 0 }, internal_file_attribute: 0, external_file_attribute: 0, extra_fields: [], comment: "" }, offset 274
[DEBUG] mod.rs: Entry: ZipEntry { filename: "decode.onnx", compression: Deflate, compression_level: Default, crc32: 1207829611, uncompressed_size: 57149888, compressed_size: 53414353, attribute_compatibility: Unix, last_modification_date: ZipDateTime { date: 0, time: 0 }, internal_file_attribute: 0, external_file_attribute: 0, extra_fields: [], comment: "" }, offset 25934
[DEBUG] mod.rs: Entry: ZipEntry { filename: "predict_duration.onnx", compression: Deflate, compression_level: Default, crc32: 4236608546, uncompressed_size: 53381, compressed_size: 48835, attribute_compatibility: Unix, last_modification_date: ZipDateTime { date: 0, time: 0 }, internal_file_attribute: 0, external_file_attribute: 0, extra_fields: [], comment: "" }, offset 53440344
[DEBUG] mod.rs: Entry: ZipEntry { filename: "manifest.json", compression: Deflate, compression_level: Default, crc32: 64471537, uncompressed_size: 220, compressed_size: 106, attribute_compatibility: Unix, last_modification_date: ZipDateTime { date: 0, time: 0 }, internal_file_attribute: 0, external_file_attribute: 0, extra_fields: [], comment: "" }, offset 53489246
[INFO] run.py: Creating an AudioQuery from 'これはテストです'
[INFO] run.py: Synthesizing with {"accent_phrases": [{"moras": [{"text": "コ", "consonant": "k", "consonant_length": 0.063058704, "vowel": "o", "vowel_length": 0.08937682, "pitch": 5.5699606}, {"text": "レ", "consonant": "r", "consonant_length": 0.047547057, "vowel": "e", "vowel_length": 0.07596417, "pitch": 5.6643105}, {"text": "ワ", "consonant": "w", "consonant_length": 0.053706698, "vowel": "a", "vowel_length": 0.10348523, "pitch": 5.7773266}], "accent": 3, "pause_mora": null, "is_interrogative": false}, {"moras": [{"text": "テ", "consonant": "t", "consonant_length": 0.06311223, "vowel": "e", "vowel_length": 0.07596652, "pitch": 5.8817406}, {"text": "ス", "consonant": "s", "consonant_length": 0.038565055, "vowel": "U", "vowel_length": 0.050694168, "pitch": 0.0}, {"text": "ト", "consonant": "t", "consonant_length": 0.06685759, "vowel": "o", "vowel_length": 0.0753997, "pitch": 5.7373238}, {"text": "デ", "consonant": "d", "consonant_length": 0.058399618, "vowel": "e", "vowel_length": 0.09201351, "pitch": 5.474717}, {"text": "ス", "consonant": "s", "consonant_length": 0.08852549, "vowel": "U", "vowel_length": 0.1281984, "pitch": 0.0}], "accent": 1, "pause_mora": null, "is_interrogative": false}], "speed_scale": 1.0, "pitch_scale": 0.0, "intonation_scale": 1.0, "volume_scale": 1.0, "pre_phoneme_length": 0.1, "post_phoneme_length": 0.1, "output_sampling_rate": 24000, "output_stereo": false, "kana": "コレワ'/テ'_ストデ_ス"}
[INFO] run.py: Wrote `audio.wav`
[DEBUG] lib.rs: Destructing a VoicevoxCore
```

正常に実行されれば音声合成の結果である wav ファイルが生成されます。
この例の場合、`"これはテストです"`という読み上げの wav ファイルが audio.wav という名前で生成されます。
