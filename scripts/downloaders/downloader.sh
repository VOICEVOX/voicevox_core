#!/usr/bin/env bash
set -eu
help(){
  cat <<EOM
  Usage: $(basename "$0") [OPTION]...
    -h|--help                      ヘルプの表示
    -o|--output \$directory        出力先の指定(default ./voicevox_core)
    -v|--version \$version         ダウンロードするvoicevox_coreのバージョンの指定(default latest)
    --artifact-type $artifact_type ダウンロードするartifact_typeを指定する(cpu,cudaを指定可能)
    --full                         DirectML版及びCUDA版ダウンロード時に追加で必要なライブラリをダウンロードする
EOM
  exit 2
}

voicevox_core_repository_base_url="https://github.com/VOICEVOX/voicevox_core"
open_jtalk_dict_url="https://jaist.dl.sourceforge.net/project/open-jtalk/Dictionary/open_jtalk_dic-1.11/open_jtalk_dic_utf_8-1.11.tar.gz"
open_jtalk_dict_dir_name="open_jtalk_dic_utf_8-1.11"

voicevox_core_releases_url(){
  os=$1
  cpu_arch=$2
  artifact_type=$3
  version=$4

  if [ "$os" = "linux" ] && [ "$artifact_type" = "cuda" ];then
    artifact_type="gpu"
  fi
  url="$voicevox_core_repository_base_url/releases/download/$version/voicevox_core-$os-$cpu_arch-$artifact_type-$version.zip"
  echo "$url"
}

latest_version(){
  base_url=$1
  get_latest_url="$base_url/releases/tag"
  echo -En $(curl -sSI "$base_url/releases/latest"| grep -oP "location: $get_latest_url/\K.*$" | sed 's/\r//g')
}

latest_voicevox_core_version(){
  latest_version $voicevox_core_repository_base_url
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
  target=$1
  url=$2
  extract_dir=$3
  archive_format="${4+x}"
  if [[ "$url" == *.tar.gz ]]; then
    archive_format="tar.gz"
  else
    archive_format="zip"
  fi

  echo "$targetを$urlからファイルをダウンロードします..."
  tmp_path=$(mktemp)
  curl -sSLo "$tmp_path" "$url"
  echo "$targetをダウンロード完了,$archive_format形式で$extract_dirに解凍します..."
  if [ "$archive_format" = "zip" ];then
    unzip -jo "$tmp_path" -d "$extract_dir"
  elif  [ "$archive_format" = "tar.gz" ];then
    mkdir -p "$extract_dir"
    tar --overwrite --strip-components 1 -xvzf "$tmp_path" -C "$extract_dir"
  fi
  echo "$targetのファイルを展開完了しました"
}

version="latest"
artifact_type=""
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
    --artifact-type)
      artifact_type="$2"
      shift;;
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
open_jtalk_output="${output%/}/$open_jtalk_dict_dir_name"


if [ "$artifact_type" = "" ];then
  artifact_type="cpu"
fi

# zipファイルに厳格なバージョン番号が含まれるため、latestだった場合はバージョンを特定して設定する
if [ "$version" = "latest" ];then
  version=$(latest_voicevox_core_version)
fi

echo "対象OS:$os"
echo "対象CPUアーキテクチャ:$cpu_arch"
echo "ダウンロードvoicevox_coreバージョン:$version"
echo "ダウンロードアーティファクトタイプ:$artifact_type"


voicevox_core_url=$(voicevox_core_releases_url "$os" "$cpu_arch" "$artifact_type" "$version")

download_and_extract "voicevox_core" "$voicevox_core_url" "$output" &
voicevox_core_download_task=$!
download_and_extract "open_jtalk" "$open_jtalk_dict_url" "$open_jtalk_output" &
open_jtalk_download_task=$!
wait $voicevox_core_download_task
wait $open_jtalk_download_task
