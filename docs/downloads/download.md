# VOICEVOX CORE Downloader

<a id="default"></a>
<a id="cpu"></a>

## デフォルト(CPU 版)をダウンロードする場合

### Windows の場合

PowerShell で下記コマンドを実行してください

```PowerShell
Invoke-WebRequest https://github.com/VOICEVOX/voicevox_core/releases/latest/download/download.ps1 -OutFile ./download.ps1
./download.ps1
```

### Linux/macOS の場合

```bash
curl -sSfL https://github.com/VOICEVOX/voicevox_core/releases/latest/download/download.sh | bash -s
```

<a id="directml"></a>

## DirectML 版をダウンロードする場合

```PowerShell
Invoke-WebRequest https://github.com/VOICEVOX/voicevox_core/releases/latest/download/download.ps1 -OutFile ./download.ps1
./download.ps1 -Device directml
```

<a id="cuda"></a>

## CUDA 版をダウンロードする場合

### Windows の場合

```PowerShell
Invoke-WebRequest https://github.com/VOICEVOX/voicevox_core/releases/latest/download/download.ps1 -OutFile ./download.ps1
./download.ps1 -Device cuda
```

### Linux の場合

```bash
curl -sSfL https://github.com/VOICEVOX/voicevox_core/releases/latest/download/download.sh | bash -s -- --device cuda
```

<a id="help"></a>

## その他詳細なオプションを指定したい場合

スクリプトにヘルプ表示機能があります。
以下のようにしてヘルプを表示できます。

### Windows の場合

```PowerShell
Invoke-WebRequest https://github.com/VOICEVOX/voicevox_core/releases/latest/download/download.ps1 -OutFile ./download.ps1
Get-Help ./download.ps1 -full
```

### Linux/macOS の場合

```bash
curl -sSfL https://github.com/VOICEVOX/voicevox_core/releases/latest/download/download.sh | bash -s -- --help
```
