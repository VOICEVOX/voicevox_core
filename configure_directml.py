import argparse
from pathlib import Path
import subprocess
import platform
import tempfile
from turtle import down


project_root = Path(__file__).absolute().parent


def run_subprocess(command):
    status, output = subprocess.getstatusoutput(command)
    if status != 0:
        raise RuntimeError(f"Failed to run: {command}\n{output}")


os_name = platform.system()
architecture_name = platform.machine().lower()


def download_and_extract_dml(link):
    if(project_root / "directml").exists():
        print(
            "Skip downloading onnxruntime because onnxruntime directory already exists."
        )
        return

    print(f"Downloading DirectML from {link}")
    with tempfile.TemporaryDirectory() as tmp_dir:
        run_subprocess(
            f'powershell -Command "cd {tmp_dir}; curl.exe {link} -L -o archive.zip"')
        run_subprocess(
            f'powershell -Command "cd {tmp_dir}; Expand-Archive -Path archive.zip; Copy-Item archive -Recurse {project_root}/directml"')


def copy_files():
    arch_type = ""
    if architecture_name in ["x86_64", "x64", "amd64"]:
        arch_type = "x64"
    elif architecture_name in ["i386", "x86"]:
        arch_type = "x86"
    elif architecture_name == "armv7l":
        arch_type = "arm"
    elif architecture_name == "aarch64":
        arch_type = "arm64"
    else:
        raise RuntimeError(
            f"Unsupported architecture type: {architecture_name}")

    dll_file_path = project_root / "directml" / \
        'bin' / f"{arch_type}-win" / "DirectML.dll"
    run_subprocess(
        f"copy /y {dll_file_path} {project_root / 'core'/'lib'/'DirectML.dll'}"
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--directml_version",
        default="1.8.0",
        help="DirectML version found in https://www.nuget.org/packages/Microsoft.AI.DirectML",
    )
    args = parser.parse_args()

    if os_name != "Windows":
        raise RuntimeError("DirectML can only be used on Windows")

    dml_url = f"https://www.nuget.org/api/v2/package/Microsoft.AI.DirectML/{args.directml_version}"

    download_and_extract_dml(dml_url)
    copy_files()

    print("Successfully configured!")
