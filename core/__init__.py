import ctypes
import sys
import os
import glob

# dll loading trick for win32.
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
    
    dlls = glob.glob(os.path.join(dll_path, '*.dll'))
    path_patched = False
    for dll in dlls:
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
            if not path_patched:
                os.environ['PATH'] = dll_path + ';'.join([os.environ['PATH']])
                path_patched = True
            res = kernel32.LoadLibraryW(dll)
            if res is None:
                err = ctypes.WinError(ctypes.get_last_error())
                err.strerror += f' Error loading "{dll}" or one of its dependencies.'
                raise err

    kernel32.SetErrorMode(prev_error_mode)


from ._core import *
 