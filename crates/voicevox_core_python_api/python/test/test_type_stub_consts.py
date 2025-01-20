"""pyiに書かれている定数の値が、本物と合致しているかをテストする。"""

import ast
from ast import AnnAssign, ClassDef, Constant, Name
from pathlib import Path

import voicevox_core


def test() -> None:
    REAL_BLOCKING = (
        voicevox_core.blocking.Onnxruntime.LIB_NAME,
        voicevox_core.blocking.Onnxruntime.LIB_VERSION,
    )
    REAL_ASYNCIO = (
        voicevox_core.asyncio.Onnxruntime.LIB_NAME,
        voicevox_core.asyncio.Onnxruntime.LIB_VERSION,
    )
    stub_blocking = extract(Path("./python/voicevox_core/_rust/blocking.pyi"))
    stub_asyncio = extract(Path("./python/voicevox_core/_rust/asyncio.pyi"))
    assert len({REAL_BLOCKING, REAL_ASYNCIO, stub_blocking, stub_asyncio}) == 1


def extract(pyi: Path) -> tuple[str, str]:
    module = ast.parse(pyi.read_text(encoding="utf-8"))
    class_def = next(
        stmt
        for stmt in module.body
        if isinstance(stmt, ClassDef) and stmt.name == "Onnxruntime"
    )
    lib_name_value = next(
        stmt.value.value
        for stmt in class_def.body
        if isinstance(stmt, AnnAssign)
        and isinstance(stmt.target, Name)
        and stmt.target.id == "LIB_NAME"
        and isinstance(stmt.value, Constant)
        and isinstance(stmt.value.value, str)
    )
    lib_version_value = next(
        stmt.value.value
        for stmt in class_def.body
        if isinstance(stmt, AnnAssign)
        and isinstance(stmt.target, Name)
        and stmt.target.id == "LIB_VERSION"
        and isinstance(stmt.value, Constant)
        and isinstance(stmt.value.value, str)
    )
    return (lib_name_value, lib_version_value)
