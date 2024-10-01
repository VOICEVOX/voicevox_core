# TODO: ここは #803 の時点でさほど必要性が無くなっているはずなので、(ドキュメントでの案内
# はした上で)やめる
import glob
import platform
from ctypes import CDLL
from pathlib import Path

if platform.system() == "Windows":
    pathname = "*.dll"
elif platform.system() == "Darwin":
    pathname = "*.dylib"
elif platform.system() == "Linux":
    pathname = "*.so.*"
else:
    raise RuntimeError("Unsupported platform")

for dll in glob.glob(pathname):
    CDLL(str(Path(dll).resolve(strict=True)))
