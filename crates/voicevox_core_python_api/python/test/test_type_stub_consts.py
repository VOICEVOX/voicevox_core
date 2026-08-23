"""pyiに書かれている定数の値が、本物と合致しているかをテストする。"""

import ast
from ast import AnnAssign, ClassDef, Constant, Name
from pathlib import Path

import voicevox_core
from voicevox_core._rust.blocking import AudioFeature


def test_audio_feature() -> None:
    REAL = (AudioFeature.FRAME_RATE,)
    stub = extract_audio_feature_const(
        Path("./python/voicevox_core/_rust/blocking.pyi"),
    )
    assert stub == REAL


def extract_audio_feature_const(pyi: Path) -> tuple[float]:
    module = ast.parse(pyi.read_text(encoding="utf-8"))
    class_def = next(
        stmt
        for stmt in module.body
        if isinstance(stmt, ClassDef) and stmt.name == "AudioFeature"
    )
    frame_rate_value = next(
        eval(ast.unparse(stmt.value))
        for stmt in class_def.body
        if isinstance(stmt, AnnAssign)
        and isinstance(stmt.target, Name)
        and stmt.target.id == "FRAME_RATE"
        and stmt.value
    )
    return (frame_rate_value,)


def test_blocking_and_asyncio() -> None:
    REAL_BLOCKING = (
        voicevox_core.blocking.Onnxruntime.LIB_MIN_REQUIRED_MINOR_VERSION,
        voicevox_core.blocking.Onnxruntime.LIB_MAX_SUPPORTED_MINOR_VERSION,
        voicevox_core.blocking.Onnxruntime.LIB_RECOMMENDED_NAME,
        voicevox_core.blocking.Onnxruntime.LIB_RECOMMENDED_VERSION,
    )
    REAL_ASYNCIO = (
        voicevox_core.asyncio.Onnxruntime.LIB_MIN_REQUIRED_MINOR_VERSION,
        voicevox_core.asyncio.Onnxruntime.LIB_MAX_SUPPORTED_MINOR_VERSION,
        voicevox_core.asyncio.Onnxruntime.LIB_RECOMMENDED_NAME,
        voicevox_core.asyncio.Onnxruntime.LIB_RECOMMENDED_VERSION,
    )
    stub_blocking = extract_onnxruntime_consts(
        Path("./python/voicevox_core/_rust/blocking.pyi"),
    )
    stub_asyncio = extract_onnxruntime_consts(
        Path("./python/voicevox_core/_rust/asyncio.pyi"),
    )
    assert len({REAL_BLOCKING, REAL_ASYNCIO, stub_blocking, stub_asyncio}) == 1


def extract_onnxruntime_consts(pyi: Path) -> tuple[int, int, str, str]:
    module = ast.parse(pyi.read_text(encoding="utf-8"))
    class_def = next(
        stmt
        for stmt in module.body
        if isinstance(stmt, ClassDef) and stmt.name == "Onnxruntime"
    )
    lib_min_required_minor_version_value = next(
        stmt.value.value
        for stmt in class_def.body
        if isinstance(stmt, AnnAssign)
        and isinstance(stmt.target, Name)
        and stmt.target.id == "LIB_MIN_REQUIRED_MINOR_VERSION"
        and isinstance(stmt.value, Constant)
        and isinstance(stmt.value.value, int)
    )
    lib_max_supported_minor_version_value = next(
        stmt.value.value
        for stmt in class_def.body
        if isinstance(stmt, AnnAssign)
        and isinstance(stmt.target, Name)
        and stmt.target.id == "LIB_MAX_SUPPORTED_MINOR_VERSION"
        and isinstance(stmt.value, Constant)
        and isinstance(stmt.value.value, int)
    )
    lib_recommended_name_value = next(
        stmt.value.value
        for stmt in class_def.body
        if isinstance(stmt, AnnAssign)
        and isinstance(stmt.target, Name)
        and stmt.target.id == "LIB_RECOMMENDED_NAME"
        and isinstance(stmt.value, Constant)
        and isinstance(stmt.value.value, str)
    )
    lib_recommended_version_value = next(
        stmt.value.value
        for stmt in class_def.body
        if isinstance(stmt, AnnAssign)
        and isinstance(stmt.target, Name)
        and stmt.target.id == "LIB_RECOMMENDED_VERSION"
        and isinstance(stmt.value, Constant)
        and isinstance(stmt.value.value, str)
    )
    return (
        lib_min_required_minor_version_value,
        lib_max_supported_minor_version_value,
        lib_recommended_name_value,
        lib_recommended_version_value,
    )
