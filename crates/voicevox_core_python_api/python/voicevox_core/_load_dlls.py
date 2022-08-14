import glob
import platform
from ctypes import CDLL
from pathlib import Path

# TODO: 実行に必要なDLLをすべてここで読む
if platform.system() == "Windows":
    filename = "onnxruntime.dll"
elif platform.system() == "Darwin":
    filename = "libonnxruntime.*.dylib"
elif platform.system() == "Linux":
    filename = "libonnxruntime.so.*"
else:
    raise RuntimeError("Unsupported platform")

for dll in glob.glob(filename):
    CDLL(str(Path(dll).resolve(strict=True)))
