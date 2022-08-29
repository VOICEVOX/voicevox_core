# VOICEVOX CORE Downloader

## 一般的な実行方法

### Windows の場合

PowerShell で下記コマンドを実行してください

```powershell
Invoke-WebRequest https://github.com/VOICEVOX/voicevox_core/releases/latest/download/Download.ps1 | powershell
```

### Linux,MacOS の場合

```bash
curl -sSLo https://github.com/VOICEVOX/voicevox_core/releases/latest/download/download.sh | bash -s
```

## DirectML 版をダウンロードする場合

一度 downloader スクリプトファイルをダウンロードする必要があります

```powershell
Invoke-WebRequest https://github.com/VOICEVOX/voicevox_core/releases/latest/download/Download.ps1 -Output ./Download.ps1
./Download.ps1 -type directml
```

## CUDA 版をダウンロードする場合

### Windows の場合

一度 downloader スクリプトファイルをダウンロードする必要があります

```powershell
Invoke-WebRequest https://github.com/VOICEVOX/voicevox_core/releases/latest/download/Download.ps1 -Output ./Download.ps1
./Download.ps1 -type cuda
```

### Linux の場合

```bash
curl -sSLo https://github.com/VOICEVOX/voicevox_core/releases/latest/download/download.sh | bash -s -- --type cuda
```

## その他詳細なオプションを指定したい場合

script の help を表示を行えば使用できる全てのオプションを参照することができます
以下に help の表示方法を示します。

### Windows の場合

一度 downloader スクリプトファイルをダウンロードする必要があります

```powershell
Invoke-WebRequest https://github.com/VOICEVOX/voicevox_core/releases/latest/download/Download.ps1 -Output ./Download.ps1
Get-Help ./Download.ps1 -full
```

### Linux,MacOS の場合

```bash
curl -sSLo https://github.com/VOICEVOX/voicevox_core/releases/latest/download/download.sh | bash -s -- --help
```
