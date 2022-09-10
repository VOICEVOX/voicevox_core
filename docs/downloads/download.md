# VOICEVOX CORE Downloader

<a id="default"></a>

## デフォルト(CPU 版)をダウンロードする場合

### Windows の場合

PowerShell で下記コマンドを実行してください

```PowerShell
Invoke-WebRequest https://github.com/VOICEVOX/voicevox_core/releases/latest/download/download.ps1 | PowerShell
```

### Linux/macOS の場合

```bash
curl -sSL https://github.com/VOICEVOX/voicevox_core/releases/latest/download/download.sh | bash -s
```

<a id="directml"></a>

## DirectML 版をダウンロードする場合

```PowerShell
Invoke-WebRequest https://github.com/VOICEVOX/voicevox_core/releases/latest/download/download.ps1 -OutFile ./download.ps1
./download.ps1 -type directml
```

<a id="cuda"></a>

## CUDA 版をダウンロードする場合

### Windows の場合

```PowerShell
Invoke-WebRequest https://github.com/VOICEVOX/voicevox_core/releases/latest/download/download.ps1 -Output ./download.ps1
./download.ps1 -type cuda
```

### Linux の場合

```bash
curl -sSL https://github.com/VOICEVOX/voicevox_core/releases/latest/download/download.sh | bash -s -- --type cuda
```

## その他詳細なオプションを指定したい場合

スクリプトにヘルプ表示機能があります。
以下のようにしてヘルプを表示できます。

### Windows の場合

```PowerShell
Invoke-WebRequest https://github.com/VOICEVOX/voicevox_core/releases/latest/download/download.ps1 -Output ./download.ps1
Get-Help ./download.ps1 -full
```

### Linux/macOS の場合

```bash
curl -sSL https://github.com/VOICEVOX/voicevox_core/releases/latest/download/download.sh | bash -s -- --help
```
