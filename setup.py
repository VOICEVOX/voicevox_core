from setuptools import setup, Extension
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

if __name__ == '__main__':
    base_dir = os.path.dirname(os.path.abspath(__file__))

    # C++モジュールがすでにビルドされ、core/libに入っているか確認
    assert os.path.exists(os.path.join(base_dir, 'core', 'lib', 'core.h')), 'C++モジュールがビルドされていません'

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

    sys.path.append(os.path.join(base_dir, 'tests'))

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
        test_suite="core_test",
    )
