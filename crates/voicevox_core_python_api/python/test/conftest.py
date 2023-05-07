import os
from pathlib import Path
import pytest
import requests
import tarfile

# onnxruntimeを最初に読み込んでおく
if ort_path := os.getenv("ORT_PATH"):
    import ctypes
    ctypes.cdll.LoadLibrary(ort_path)

root_dir = Path(os.path.dirname(os.path.abspath(__file__)))


@pytest.fixture(scope="session")
def open_jtalk_dict_dir():
    if os.path.exists(root_dir / "open_jtalk_dic_utf_8-1.11"):
        return root_dir / "open_jtalk_dic_utf_8-1.11"
    response = requests.get(
        "https://github.com/r9y9/open_jtalk/releases/download/v1.11.1/open_jtalk_dic_utf_8-1.11.tar.gz"
    )
    response.raise_for_status()
    with open(root_dir / "open_jtalk_dic_utf_8-1.11.tar.gz", "wb") as f:
        f.write(response.content)
    with tarfile.open(root_dir / "open_jtalk_dic_utf_8-1.11.tar.gz") as tar:
        tar.extractall(root_dir)
    os.remove(root_dir / "open_jtalk_dic_utf_8-1.11.tar.gz")
    return root_dir / "open_jtalk_dic_utf_8-1.11"

@pytest.fixture(scope="session")
def core(open_jtalk_dict_dir):
    import voicevox_core
    core = voicevox_core.VoicevoxCore(open_jtalk_dict_dir=open_jtalk_dict_dir)
    core.load_model(0)
    return core

