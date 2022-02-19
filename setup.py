# -*- coding: utf-8 -*-

from setuptools import setup
import platform
import os
import sys


def get_version():
    """バージョン取得"""
    base_dir = os.path.dirname(os.path.abspath(__file__))
    with open(os.path.join(base_dir, 'VERSION.txt')) as f:
        return f.read().strip()


if __name__ == '__main__':
    base_dir = os.path.dirname(os.path.abspath(__file__))

    # C++モジュールがすでにビルドされ、core/libに入っているか確認
    assert os.path.exists(os.path.join(
        base_dir, 'core', 'lib', 'core.h')), 'C++モジュールがビルドされていません'

    sys.path.append(os.path.join(base_dir, 'tests'))

    setup(
        name="core",
        version=get_version(),
        packages=["core"],
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
