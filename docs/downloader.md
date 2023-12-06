# VOICEVOX CORE Downloader
VOICEVOX COREの実行には音声モデル（VVM ファイル）やOpen JTalkなどの外部ライブラリのダウンロードが必要になります。
VOICEVOX CORE Downloaderは環境に合わせてそれらをダウンロードします。

> [!NOTE]
> 音声モデル（VVM ファイル）には利用規約が存在します。詳しくはダウンロードしたファイル内の README に記載されています。

# ダウンローダーの入手

### Windows の場合

PowerShell で下記コマンドを実行してください

```PowerShell
Invoke-WebRequest https://github.com/VOICEVOX/voicevox_core/releases/latest/download/download-windows-x64.exe -OutFile ./download.exe
```

### Linux/macOS の場合

[最新のリリース](https://github.com/VOICEVOX/voicevox_core/releases/latest)から環境に合わせてダウンローダーのバイナリをダウンロードしてください。
現在利用可能なのは以下の4つです。

* download-linux-arm64
* download-linux-x64
* download-osx-arm64
* download-osx-x64

以下はLinuxのx64での実行例です。

```bash
binary=download-linux-x64
curl -sSfL https://github.com/VOICEVOX/voicevox_core/releases/latest/download/${binary} -o download
chmod +x download
```

# ダウンローダーの使い方


<a id="default"></a>
<a id="cpu"></a>

## デフォルト(CPU 版)をダウンロードする場合


```
download
```

または

```
download --device cpu
```

<a id="directml"></a>

## DirectML 版をダウンロードする場合

```
download --device directml
```

<a id="cuda"></a>

## CUDA 版をダウンロードする場合

```
download --device cuda
```

<a id="help"></a>

## その他詳細なオプションを指定したい場合

スクリプトにヘルプ表示機能があります。
以下のようにしてヘルプを表示できます。

```
download --help
```
