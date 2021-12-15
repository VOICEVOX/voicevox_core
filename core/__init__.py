import ctypes
import sys
import os
import glob

"""dll loading trick for win32.
This makes win32 dlls loadable from the folder outside of the DLL search path.

Original code from: https://github.com/pytorch/pytorch/blob/db014b8529984df908c9f941eaf8c68ace31507a/torch/__init__.py#L55
Whole copyright notice of the original code is below.
-------------------------------------------------------------------------------
From PyTorch:

Copyright (c) 2016-     Facebook, Inc            (Adam Paszke)
Copyright (c) 2014-     Facebook, Inc            (Soumith Chintala)
Copyright (c) 2011-2014 Idiap Research Institute (Ronan Collobert)
Copyright (c) 2012-2014 Deepmind Technologies    (Koray Kavukcuoglu)
Copyright (c) 2011-2012 NEC Laboratories America (Koray Kavukcuoglu)
Copyright (c) 2011-2013 NYU                      (Clement Farabet)
Copyright (c) 2006-2010 NEC Laboratories America (Ronan Collobert, Leon Bottou, Iain Melvin, Jason Weston)
Copyright (c) 2006      Idiap Research Institute (Samy Bengio)
Copyright (c) 2001-2004 Idiap Research Institute (Ronan Collobert, Samy Bengio, Johnny Mariethoz)

From Caffe2:

Copyright (c) 2016-present, Facebook Inc. All rights reserved.

All contributions by Facebook:
Copyright (c) 2016 Facebook Inc.

All contributions by Google:
Copyright (c) 2015 Google Inc.
All rights reserved.

All contributions by Yangqing Jia:
Copyright (c) 2015 Yangqing Jia
All rights reserved.

All contributions by Kakao Brain:
Copyright 2019-2020 Kakao Brain

All contributions from Caffe:
Copyright(c) 2013, 2014, 2015, the respective contributors
All rights reserved.

All other contributions:
Copyright(c) 2015, 2016 the respective contributors
All rights reserved.

Caffe2 uses a copyright model similar to Caffe: each contributor holds
copyright over their contributions to Caffe2. The project versioning records
all such contribution and copyright details. If a contributor wants to further
mark their specific copyright on a particular contribution, they should
indicate their copyright solely in the commit message of the change when it is
committed.

All rights reserved.

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are met:

1. Redistributions of source code must retain the above copyright
   notice, this list of conditions and the following disclaimer.

2. Redistributions in binary form must reproduce the above copyright
   notice, this list of conditions and the following disclaimer in the
   documentation and/or other materials provided with the distribution.

3. Neither the names of Facebook, Deepmind Technologies, NYU, NEC Laboratories America
   and IDIAP Research Institute nor the names of its contributors may be
   used to endorse or promote products derived from this software without
   specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT OWNER OR CONTRIBUTORS BE
LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE
POSSIBILITY OF SUCH DAMAGE.
"""
if sys.platform == 'win32':
    kernel32 = ctypes.WinDLL('kernel32.dll', use_last_error=True)
    with_load_library_flags = hasattr(kernel32, 'AddDllDirectory')
    prev_error_mode = kernel32.SetErrorMode(0x0001)

    kernel32.LoadLibraryW.restype = ctypes.c_void_p
    if with_load_library_flags:
        kernel32.AddDllDirectory.restype = ctypes.c_void_p
        kernel32.LoadLibraryExW.restype = ctypes.c_void_p

    dll_path = os.path.join(os.path.dirname(__file__), 'lib')
    if sys.version_info >= (3, 8):
        os.add_dll_directory(dll_path)
    elif with_load_library_flags:
        res = kernel32.AddDllDirectory(dll_path)
        if res is None:
            err = ctypes.WinError(ctypes.get_last_error())
            err.strerror += f' Error adding "{dll_path}" to the DLL directories.'
            raise err

    # 明示的にcore.dllを読み込めば、onnxruntimeなどの残りの依存は自動で解決してくれる
    # Note: onnxruntime_providers_cuda.dllはLoadLibraryによってロードしようとすると失敗する (GitHub PR #49)
    dll = os.path.join(dll_path, 'core.dll')
    is_loaded = False
    if with_load_library_flags:
        res = kernel32.LoadLibraryExW(dll, None, 0x00001100)
        last_error = ctypes.get_last_error()
        if res is None and last_error != 126:
            err = ctypes.WinError(last_error)
            err.strerror += f' Error loading "{dll}" or one of its dependencies.'
            raise err
        elif res is not None:
            is_loaded = True
    if not is_loaded:
        os.environ['PATH'] = ';'.join([dll_path] + [os.environ['PATH']])
        res = kernel32.LoadLibraryW(dll)
        if res is None:
            err = ctypes.WinError(ctypes.get_last_error())
            err.strerror += f' Error loading "{dll}" or one of its dependencies.'
            raise err

    kernel32.SetErrorMode(prev_error_mode)

# load the core library
from ._core import *
 