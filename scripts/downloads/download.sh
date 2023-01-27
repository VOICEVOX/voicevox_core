#!/usr/bin/env bash
set -eu
help(){
  cat <<EOM
  Usage: $(basename "$0") [OPTION]...
    -h|--help                                 ヘルプの表示
    -o|--output \$directory                    出力先の指定(default ./voicevox_core)
    -v|--version \$version                     ダウンロードするvoicevox_coreのバージョンの指定(default latest)
    --additional-libraries-version \$version   追加でダウンロードするライブラリのバージョン
    --accelerator \$accelerator                ダウンロードするacceleratorを指定する(cpu,cudaを指定可能.cudaはlinuxのみ)
    --cpu-arch \$cpu_arch                      ダウンロードするcpuのアーキテクチャを指定する
    --min                                     ダウンロードするライブラリを最小限にするように指定
    --os                                      ダウンロードする対象のOSを指定する
EOM
  exit 2
}

voicevox_core_repository_base_url="https://github.com/VOICEVOX/voicevox_core"
voicevox_additional_libraries_base_url="https://github.com/VOICEVOX/voicevox_additional_libraries"
open_jtalk_dict_url="https://jaist.dl.sourceforge.net/project/open-jtalk/Dictionary/open_jtalk_dic-1.11/open_jtalk_dic_utf_8-1.11.tar.gz"
open_jtalk_dict_dir_name="open_jtalk_dic_utf_8-1.11"

voicevox_core_releases_url(){
  os=$1
  cpu_arch=$2
  accelerator=$3
  version=$4

  if [ "$os" = "linux" ] && [ "$accelerator" = "cuda" ];then
    accelerator="gpu"
  fi
  url="$voicevox_core_repository_base_url/releases/download/$version/voicevox_core-$os-$cpu_arch-$accelerator-$version.zip"
  echo "$url"
}

voicevox_additional_libraries_url(){
  os=$1
  cpu_arch=$2
  accelerator=$3
  version=$4
  if [ "$accelerator" = "cuda" ];then
    accelerator="CUDA"
  elif [ "$accelerator" = "directml" ];then
    accelerator="DirectML"
  fi
  url="${voicevox_additional_libraries_base_url}/releases/download/${version}/${accelerator}-${os}-${cpu_arch}.zip"
  echo "$url"
}

latest_version(){
  base_url=$1
  get_latest_url="$base_url/releases/tag"
  echo -En "$(curl -sSfI "$base_url/releases/latest"| grep "location:" | sed -e "s%location: $get_latest_url/%%" | sed 's/\r//g')"
}

latest_voicevox_core_version(){
  latest_version $voicevox_core_repository_base_url
}

latest_voicevox_additional_libraries_version(){
  latest_version $voicevox_additional_libraries_base_url
}

target_os(){
  if [ "$(uname)" == 'Darwin' ]; then
    echo "osx"
  elif [[ "$(uname)" =~ Linux ]]; then
    echo "linux"
  else
    echo "$(uname)はサポートされていない環境です" >&2
    exit 1
  fi
}

target_arch(){
  cpu_arch=$(uname -m)
  case "$cpu_arch" in
    "x86_64") echo "x64";;
    "arm64") echo "arm64";;
    *)
      echo "$cpu_archはサポートされていない環境です" >&2
      exit 1;;
  esac
}

download_and_extract(){
  target=$1
  url=$2
  extract_dir=$3
  archive_format="${4+x}"
  if [[ "$archive_format" == "" ]]; then
    if [[ "$url" == *.tar.gz ]]; then
      archive_format="tar.gz"
    else
      archive_format="zip"
    fi
  fi

  echo "${target}を${url}からファイルをダウンロードします..."
  tmp_path=$(mktemp)
  curl -sSLfo "$tmp_path" "$url"
  echo "${target}をダウンロード完了,${archive_format}形式で${extract_dir}に解凍します..."
  if [ "$archive_format" = "zip" ];then
    top_dir=$(unzip -Z1 "$tmp_path" | head -n 1)
    unzip "$tmp_path" -d "$extract_dir"
    mv "$extract_dir/$top_dir"/* "$extract_dir"
    rmdir "$extract_dir/$top_dir"
  elif  [ "$archive_format" = "tar.gz" ];then
    mkdir -p "$extract_dir"
    tar --strip-components 1 -xvzf "$tmp_path" -C "$extract_dir"
  fi
  echo "${target}のファイルを展開完了しました。後続のファイルダウンロード処理を待ってください"
}

os=""
cpu_arch=""
version="latest"
additional_libraries_version="latest"
accelerator=""
output="./voicevox_core"
min=""




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
    --additional-libraries-version)
      additional_libraries_version="$2"
      shift;;
    --accelerator)
      accelerator="$2"
      shift;;
    --cpu-arch)
      cpu_arch="$2"
      shift;;
    --min)
      min=true
      ;;
    --os)
      os="$2"
      shift;;
    *)
      echo "サポートされていないオプションです" >&2
      exit 1;;
  esac
  shift
done

open_jtalk_output="${output%/}/$open_jtalk_dict_dir_name"

if [ -z "$os" ];then
  os=$(target_os)
fi

if [ -z "$cpu_arch" ];then
  cpu_arch=$(target_arch)
fi

if [ "$accelerator" = "" ];then
  accelerator="cpu"
fi

if [ "$accelerator" = "cpu" ];then
  additional_libraries_version=""
fi

# zipファイルに厳格なバージョン番号が含まれるため、latestだった場合はバージョンを特定して設定する
if [ "$version" = "latest" ];then
  version=$(latest_voicevox_core_version)
fi

if [ "$additional_libraries_version" != "" ] && [ "$additional_libraries_version" = "latest" ];then
  additional_libraries_version=$(latest_voicevox_additional_libraries_version)
fi

echo "対象OS:$os"
echo "対象CPUアーキテクチャ:$cpu_arch"
echo "ダウンロードvoicevox_coreバージョン:$version"
echo "ダウンロードアーティファクトタイプ:$accelerator"

if [ "$additional_libraries_version" != "" ];then
  echo "ダウンロード追加ライブラリバージョン:$additional_libraries_version"
fi


voicevox_core_url=$(voicevox_core_releases_url "$os" "$cpu_arch" "$accelerator" "$version")
voicevox_additional_libraries_url=$(voicevox_additional_libraries_url "$os" "$cpu_arch" "$accelerator" "$additional_libraries_version")

download_and_extract "voicevox_core" "$voicevox_core_url" "$output" &
voicevox_core_download_task=$!
if [ "$min" != "true" ]; then
  download_and_extract "open_jtalk" "$open_jtalk_dict_url" "$open_jtalk_output" &
  open_jtalk_download_task=$!

  if [ "$additional_libraries_version" != "" ];then
    download_and_extract "voicevox_additional_libraries" "$voicevox_additional_libraries_url" "$output" &
    additional_libraries_download_task=$!
    wait $additional_libraries_download_task
  fi

  wait $open_jtalk_download_task
fi
wait $voicevox_core_download_task

echo "全ての必要なファイルダウンロードが完了しました"
