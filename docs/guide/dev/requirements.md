> [!IMPORTANT]
> このページは未完成です。何かおかしな点があればお気軽に指摘してください。

# 構成要件

VOICEVOX CORE の開発に必要なツールとライブラリの一覧です。
ビルド前にインストールしてください。

## 開発ツール

### 共通

| ツール | インストール方法 / リンク |
| :--- | :--- |
| **Rust** | [公式サイト](https://www.rust-lang.org/tools/install) (`rustup`) |
| **Git** | [公式サイト](https://git-scm.com/downloads) |
| **CMake** | [公式サイト](https://cmake.org/download/) |
| **jq** | [公式サイト](https://stedolan.github.io/jq/download/) |
| **ONNX Runtime** | ビルドスクリプトにより用意されます (`cargo run -p downloader`) |

### 言語別追加要件

- **Python**: [Poetry](https://python-poetry.org/docs/#installation), `pip install pyflakes`
- **Java**: [Adoptium (JDK 11/17)](https://adoptium.net/), [Gradle](https://gradle.org/install/) (またはリポジトリの `gradlew`)

## OS 別インストールコマンド例

### Ubuntu / Debian

```bash
sudo apt update
sudo apt install git cmake jq build-essential shellcheck clang
```

### macOS

```bash
# Command Line Tools のインストール
xcode-select --install
# Homebrew を使用する場合
brew install cmake jq
```

### Windows

1. [Visual Studio](https://visualstudio.microsoft.com/ja/vs/features/cplusplus/) をインストールし、「C++ によるデスクトップ開発」ワークロードを選択。
2. [7-Zip](https://www.7-zip.org/) をインストール。
