from setuptools import setup, Extension
from subprocess import check_call, CalledProcessError
import platform
import os
import sys

import numpy
from Cython.Build import cythonize
from Cython.Distutils import build_ext

def get_version():
    """バージョン取得"""
    base_dir = os.path.dirname(os.path.abspath(__file__))
    with open(os.path.join(base_dir, 'VERSION.txt')) as f:
        return f.read().strip()

def build_src():
    """setupより前にC++モジュールのビルド"""
    print('Building C++ modules...')

    base_dir = os.path.dirname(os.path.abspath(__file__))
    build_dir = os.path.join(base_dir, 'build')
    os.makedirs(build_dir, exist_ok=True)
    try:
        check_call(['cmake', '..'], cwd=build_dir)
        check_call(['cmake', '--build', '.', '--config', 'Release'], cwd=build_dir)
        check_call(['cmake', '--install', '.'], cwd=build_dir)
    except (CalledProcessError, KeyboardInterrupt) as e:
        sys.exit(1)

if __name__ == '__main__':
    build_src()

    # 追加ライブラリ(pythonライブラリからの相対パスで./lib/*)を読み込めるように設定
    if platform.system() == "Windows":
        # Windowsでは別途__init__.pyで明示的に読み込む
        runtime_library_dirs = []
    else:
        # $ORIGINはpythonライブラリの読み込み時に自動的に自身のパスに展開される
        runtime_library_dirs = ["$ORIGIN/lib"]

    ext_modules = [
        Extension(
            name="core._core",
            sources=["core/_core.pyx"],
            language="c++",
            libraries=["core"],
            include_dirs=["core/lib"],
            library_dirs=["core/lib"],
            runtime_library_dirs=runtime_library_dirs,
        )
    ]

    setup(
        name="core",
        version=get_version(),
        packages=["core"],
        cmdclass={
            "build_ext": build_ext,
        },
        ext_modules=cythonize(ext_modules),
        include_dirs=[
            numpy.get_include(),
        ],
        package_data={
            "core": [
                "lib/*.dll",
                "lib/*.so",
                "lib/*.so.*",
                "lib/*.dylib"
            ]
        },
    )
