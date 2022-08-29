# VOICEVOX CORE Downloader

## デフォルト(CPU 版)をダウンロードする場合

### Windows の場合

PowerShell で下記コマンドを実行してください

```PowerShell
Invoke-WebRequest https://github.com/VOICEVOX/voicevox_core/releases/latest/download/Download.ps1 | PowerShell
```

### Linux/macOS の場合

```bash
curl -sSLo https://github.com/VOICEVOX/voicevox_core/releases/latest/download/download.sh | bash -s
```

## DirectML 版をダウンロードする場合

```PowerShell
Invoke-WebRequest https://github.com/VOICEVOX/voicevox_core/releases/latest/download/Download.ps1 -Output ./Download.ps1
./Download.ps1 -type directml
```

## CUDA 版をダウンロードする場合

### Windows の場合

```PowerShell
Invoke-WebRequest https://github.com/VOICEVOX/voicevox_core/releases/latest/download/Download.ps1 -Output ./Download.ps1
./Download.ps1 -type cuda
```

### Linux の場合

```bash
curl -sSLo https://github.com/VOICEVOX/voicevox_core/releases/latest/download/download.sh | bash -s -- --type cuda
```

## その他詳細なオプションを指定したい場合

スクリプトにヘルプ表示機能があります。
以下のようにしてヘルプを表示できます。

### Windows の場合

```PowerShell
Invoke-WebRequest https://github.com/VOICEVOX/voicevox_core/releases/latest/download/Download.ps1 -Output ./Download.ps1
Get-Help ./Download.ps1 -full
```

### Linux/macOS の場合

```bash
curl -sSLo https://github.com/VOICEVOX/voicevox_core/releases/latest/download/download.sh | bash -s -- --help
```
