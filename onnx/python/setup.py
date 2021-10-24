from setuptools import setup, Extension
import platform

import numpy
from Cython.Build import cythonize
from Cython.Distutils import build_ext

if platform.system() == "Windows":
    runtime_lib_dirs = []
else:
    runtime_library_dirs = ["$ORIGIN"]

ext_modules = [
    Extension(
        name="core._core",
        sources=["core/_core.pyx"],
        language="c++",
        libraries=["core"],
        include_dirs=["core"],
        library_dirs=["core"],
        runtime_lib_dirs=runtime_lib_dirs,
    )
]

setup(
    name="core",
    packages=["core"],
    cmdclass={
        "build_ext": build_ext,
    },
    ext_modules=cythonize(ext_modules),
    include_dirs=[
        numpy.get_include(),
    ],
    package_data={"core": ["*.dll", "*.lib", "*.so", "*.so.*"]},
)
