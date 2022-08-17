#!/usr/bin/env bash
set -eu
help(){
  cat <<EOM
  Usage: $(basename "$0") [OPTION]...
    -h|--help                     ヘルプの表示
    -o|--output \$directory       出力先の指定(default ./voicevox_core)
    -v|--version \$version        ダウンロードするvoicevox_coreのバージョンの指定(default latest)
    --directml                    DirectML版voicevox_coreをダウンロードする
    --cuda                        CUDA版voicevox_coreをダウンロードする
    --cpu                         CPU版voicevox_coreをダウンロードする
    --full                        DirectML版及びCUDA版ダウンロード時に追加で必要なライブラリをダウンロードする
EOM
  exit 2
}

base_url="https://github.com/VOICEVOX/voicevox_core"

voicevox_core_releases_url(){
  os=$1
  cpu_arch=$2
  artifact_type=$3
  version=$4

  if [ "$os" = "linux" ] && [ "$artifact_type" = "cuda" ];then
    artifact_type="gpu"
  fi
  url="$base_url/releases/download/$version/voicevox_core-$os-$cpu_arch-$artifact_type-$version.zip"
  echo "$url"
}

latest_voicevox_core_version(){
  get_latest_url="$base_url/releases/tag"
  echo -En $(curl -sSI "$base_url/releases/latest"| grep -oP "location: $get_latest_url/\K.*$" | sed 's/\r//g')
}

target_os(){
  if [ "$(uname)" == 'Darwin' ]; then
    echo "osx"
  elif [ "$(expr substr $(uname -s) 1 5)" == 'Linux' ]; then
    echo "linux"
  else
    echo "サポートされていない環境です"
    exit 1
  fi
}

target_arch(){
  case "$(arch)" in
    "x86_64") echo "x64";;
    "arm64") echo "aarch64";;
    *)
      echo "サポートされていない環境です"
      exit 1;;
  esac
}

download_and_extract(){
  url=$1
  extract_dir=$2
  echo "$urlからファイルをダウンロードして$extract_dirに展開します..."
  tmp_path=$(mktemp)
  curl -sSLo "$tmp_path" "$url"
  unzip -jo "$tmp_path" -d "$extract_dir"
}

version="latest"
artifact_type="cpu"
output="./voicevox_core"
full=""




while [ $# -gt 0 ];
do
  case $1 in
    -h|--help ) help;;
    -o|--output) 
      output="$2"
      shift;;
    -v|--version)
      version="$2"
      shift;;
    --directml)
      artifact_type="directml"
      ;;
    --cuda)
      artifact_type="cuda"
      ;;
    --cpu)
      artifact_type="cpu"
      ;;
    --full)
      full=true
      ;;
    *)
      echo "サポートされていないオプションです"
      exit 1;;
  esac
  shift
done


os=$(target_os)
cpu_arch=$(target_arch)


# zipファイルに厳格なバージョン番号が含まれるため、latestだった場合はバージョンを特定して設定する
if [ "$version" = "latest" ];then
  version=$(latest_voicevox_core_version)
fi

echo "対象OS:$os"
echo "対象CPUアーキテクチャ:$cpu_arch"
echo "ダウンロードバージョン:$version"
echo "ダウンロードアーティファクトタイプ:$artifact_type"


voicevox_core_url=$(voicevox_core_releases_url "$os" "$cpu_arch" "$artifact_type" "$version")

download_and_extract "$voicevox_core_url" "$output"
