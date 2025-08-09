# VOICEVOX CORE Downloader
VOICEVOX COREの実行には音声モデル（VVM ファイル）やOpen JTalkなどの外部ライブラリのダウンロードが必要になります。
VOICEVOX CORE Downloaderは環境に合わせてそれらをダウンロードします。

> [!NOTE]
> 音声モデル（VVM ファイル）には利用規約が存在します。詳しくはダウンロードしたファイル内の README に記載されています。

## ダウンローダーがダウンロードするもの

`--only`や`--exclude`で特に指定しない場合、ダウンローダーは次のすべてをダウンロードします。

| 名称 | 展開先 | 説明 |
| :- | :- | :- |
| `c-api` | {output}/c_api/ | VOICEVOX CORE C APIのビルド済みバイナリおよびその利用規約ファイル等 |
| `onnxruntime` | {output}/onnxruntime/ | (VOICEVOX) ONNX Runtime |
| `additional-libraries` | {output}/additional_libraries/ | `--devices`で指定したDirectMLやCUDA |
| `models` | {output}/models/ | VOICEVOX音声モデル（VVMファイル） |
| `dict` | {output}/dict/ | Open JTalkのシステム辞書 |

{output}は`-o, --output`で指定したディレクトリで、デフォルトは`./voicevox_core/`です。

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
download --devices cpu
```

<a id="directml"></a>

## DirectML 版をダウンロードする場合

```
download --devices directml
```

<a id="cuda"></a>

## CUDA 版をダウンロードする場合

```
download --devices cuda
```

<a id="models-pattern"></a>

## 一部の音声モデル（VVMファイル）だけダウンロードする場合

```bash
download --models-pattern 0.vvm # 0.vvmのみダウンロード
```

```bash
download --models-pattern '[0-9]*.vvm' # トーク用VVMに絞り、ソング用VVMをダウンロードしないように
```

<a id="github-authentication-token"></a>

## レートリミットを回避する

もしGitHubのレートリミットによるエラーが発生する場合は、環境変数`GH_TOKEN`または`GITHUB_TOKEN`でGitHubの認証トークンを設定してください。認証トークンを設定することでGithubのレートリミットが緩和されます。

```bash
GH_TOKEN=$(gh auth token) download …
```

<a id="help"></a>

## その他詳細なオプションを指定したい場合

スクリプトにヘルプ表示機能があります。
以下のようにしてヘルプを表示できます。

```
download --help
```
