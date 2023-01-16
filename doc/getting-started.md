# 0.13.3をpythonで動かすまでのセットアップ

## 前提条件
[python](https://www.python.org/downloads/)をインストールしてpipも使えるようにしてください

linuxではunzipも必要です。

debian系
```
apt install unzip
```

redhat系
```
yum install unzip
```

## サンプルを動かすまで

1. 作業用ディレクトリを作る
```
mkdir work
cd work
```

2. ソース一式もってくる

```
git clone https://github.com/VOICEVOX/voicevox_core.git source
```

3. ダウンロードスクリプトの退避（[https://github.com/VOICEVOX/voicevox_core/blob/main/docs/downloads/download.md](手順)を厳格にやる場合はスキップ）
```
cp source/scripts/downloads/download.sh .
```

4. 0.13.3へ戻す
```
cd source
git checkout refs/tags/0.13.3
```

5. pythonに必要なパッケージのインストール
```
pip install -r requirements.txt
```

6. サンプルを動かすのに必要なスクリプトのコピー
```
cd ..
cp source/example/python/run.py .
cp -r source/core/ .
```

7. ダウンロードスクリプトの実行
[こちらも参考](https://github.com/VOICEVOX/voicevox_core/blob/main/docs/downloads/download.md)
```
./download.sh
```

8. ダウンロードしたライブラリの配置
```
mkdir core/lib
cp voicevox_core/core.h core/lib
cp voicevox_core/libcore.so core/lib
cp -r voicevox_core/open_jtalk_dic_utf_8-1.11/ .
```

9. onnxの配置
```
curl -LO https://github.com/microsoft/onnxruntime/releases/download/v1.13.1/onnxruntime-linux-x64-1.13.1.tgz
tar xf onnxruntime-linux-x64-1.13.1.tgz
cp onnxruntime-linux-x64-1.13.1/lib/libonnxruntime.so.1.13.1 core/lib
```

10. 実行
python run.py --text TEST --speaker_id 1
