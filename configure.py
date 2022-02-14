import argparse
import json
import subprocess
import tempfile
from glob import glob
from pathlib import Path
from typing import List, Union
from urllib import request

project_root = Path(__file__).absolute().parent


def run_subprocess(command):
    status, output = subprocess.getstatusoutput(command)
    if status != 0:
        raise RuntimeError(f"Failed to run: {command}\n{output}")


def get_os() -> str:
    return subprocess.getoutput("uname -s")


def get_arch() -> str:
    return subprocess.getoutput("uname -m")


def get_release(url: str, version: str):
    resp = request.urlopen(url)
    releases = json.loads(resp.read())
    target_release = None
    for release in releases:
        if release["tag_name"] == version:
            assert (
                target_release is None
            ), f"Multiple releases were found with tag_name: {version}."
            target_release = release
    if target_release is None:
        raise RuntimeError(f"No release was found with version: {version}.")
    return target_release


def get_ort_download_link(version: str, use_gpu: bool) -> str:
    target_release = get_release(
        "https://api.github.com/repos/microsoft/onnxruntime/releases", version
    )
    assets = target_release["assets"]
    assert assets

    def filter_assets(keywords: Union[str, List[str]]) -> None:
        if isinstance(keywords, str):
            keywords = [keywords]
        nonlocal assets
        new_assets = []
        for asset in assets:
            for keyword in keywords:
                if keyword in asset["name"]:
                    new_assets.append(asset)
                    break
        if not new_assets:
            raise RuntimeError(
                f"Asset was not found. Most likely the version is too old or new that {__file__} does not support. "
                "Try `--ort_download_link` option to manually specify the download link."
            )
        assets = new_assets

    os_type = get_os()
    arch_type = get_arch()
    if "x86_64" == arch_type:
        arch_type = ["x64", "x86_64"]
    if os_type == "Darwin":
        if use_gpu:
            raise RuntimeError("onnxruntime for osx does not support gpu.")
        filter_assets("osx")
        filter_assets(arch_type)
    elif os_type == "Linux":
        filter_assets("linux")
        filter_assets(arch_type)
        if use_gpu:
            filter_assets("gpu")
    else:
        raise RuntimeError(f"Unsupported os type: {os_type}.")
    assets = sorted(assets, key=lambda x: x["name"])
    return assets[0]["browser_download_url"]


def download_and_extract_ort(download_link):
    if (project_root / "onnxruntime").exists():
        print(
            "Skip downloading onnxruntime because onnxruntime directory already exists."
        )
        return
    print(f"Downloading onnxruntime from {download_link}...")
    with tempfile.TemporaryDirectory() as tmp_dir:
        run_subprocess(f"cd {tmp_dir} && wget {download_link} -O archive")
        extract_cmd = "unzip" if download_link.endswith(".zip") else "tar xzf"
        run_subprocess(
            f"cd {tmp_dir} && {extract_cmd} archive && cp -r onnxruntime* {project_root}/onnxruntime"
        )


def get_voicevox_download_link(version) -> str:
    target_release = get_release(
        "https://api.github.com/repos/VOICEVOX/voicevox_core/releases", version
    )
    assets = target_release["assets"]
    for asset in assets:
        if asset["name"] == "core.zip":
            return asset["browser_download_url"]
    raise RuntimeError(
        f"Asset was not found. Most likely the version is too old or new that {__file__} does not support. "
        "Try `--voicevox_download_link` option to manually specify the download link."
    )


def download_and_extract_voicevox(download_link):
    if (project_root / "release").exists():
        print(
            "Skip downloading voicevox release because release directory already exists."
        )
        return
    print(f"Downloading voicevox from {download_link}...")
    with tempfile.TemporaryDirectory() as tmp_dir:
        run_subprocess(
            f"cd {tmp_dir} && wget {download_link} && unzip core.zip && cp -r core {project_root}/release"
        )


def link_files():
    os_type = get_os()
    lib_prefix = ""
    lib_suffix = ""
    if os_type == "Darwin":
        lib_prefix = "lib"
        lib_suffix = ".dylib"
    elif os_type == "Linux":
        lib_prefix = "lib"
        lib_suffix = ".so"
    else:
        raise RuntimeError(f"Unsupported os type: {os_type}.")

    core_libs = glob(f"{project_root}/release/{lib_prefix}*{lib_suffix}")
    assert core_libs

    target_core_lib = None

    if len(core_libs) == 1:
        target_core_lib = core_libs[0]
    else:
        # TODO: Use better name so that we can decide from os_type or arch_type.
        print("Please enter number to select which library to use.")
        for i, name in enumerate(core_libs):
            print(f"{i}: {name}")
        index = input("Your choice: ")
        while not index.isdigit():
            index = input("Please enter number")
        index = int(index)
        target_core_lib = core_libs[index]

    run_subprocess(f"mkdir {project_root}/core/lib")
    run_subprocess(f"ln -s {project_root}/release/core.h {project_root}/core/lib")
    run_subprocess(f"ln -s {target_core_lib} {project_root}/core/lib/{lib_prefix}core{lib_suffix}")

    ort_libs = glob(f"{project_root}/onnxruntime/lib/{lib_prefix}*{lib_suffix}")
    assert ort_libs
    for ort_lib in ort_libs:
        run_subprocess(f"ln -s {ort_lib} {project_root}/core/lib")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--voicevox_version",
        default="0.10.0",
        help="voicevox release tag found in https://github.com/VOICEVOX/voicevox_core/releases",
    )
    parser.add_argument("--voicevox_download_link", help="voicevox download link")
    parser.add_argument(
        "--ort_version",
        default="v1.10.0",
        help="onnxruntime release tag found in https://github.com/microsoft/onnxruntime/releases",
    )
    parser.add_argument(
        "--use_gpu", action="store_true", help="enable gpu for onnxruntime"
    )
    parser.add_argument("--ort_download_link", help="onnxruntime download link")

    args = parser.parse_args()
    ort_download_link = args.ort_download_link
    if not ort_download_link:
        ort_download_link = get_ort_download_link(args.ort_version, args.use_gpu)

    download_and_extract_ort(ort_download_link)

    voicevox_download_link = args.voicevox_download_link
    if not voicevox_download_link:
        voicevox_download_link = get_voicevox_download_link(args.voicevox_version)
    download_and_extract_voicevox(voicevox_download_link)

    lib_path = project_root / "core/lib"
    if lib_path.exists():
        yn = input("Found existing library at core/lib/. Overwrite? [yn]: ")
        while yn != "y" and yn != "n":
            yn = input("Please press y or n: ")
        if yn == "n":
            exit()
        subprocess.getoutput(f"rm -r {lib_path}")

    link_files()
    print("Successfully configured!")
