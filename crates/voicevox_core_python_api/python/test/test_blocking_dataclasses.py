"""
型が壊れたデータクラスのインスタンスを生み出していないかをテストする。

``test_asyncio_dataclasses`` と対になる。
"""

import dataclasses
import typing
from types import UnionType
from typing import Literal, NewType, Union

import conftest
import pytest
from voicevox_core import StyleId, UserDictWord
from voicevox_core.blocking import Onnxruntime, OpenJtalk, Synthesizer, VoiceModelFile


def test_supported_devices(onnxruntime: Onnxruntime) -> None:
    typecheck_dataclass(onnxruntime.supported_devices())


def test_metas(model: VoiceModelFile, synthesizer: Synthesizer) -> None:
    for meta in model.metas:
        typecheck_dataclass(meta)
    for meta in synthesizer.metas():
        typecheck_dataclass(meta)


def test_audio_query(open_jtalk: OpenJtalk, synthesizer: Synthesizer) -> None:
    TEXT = "こんにちは"
    STYLE_ID = StyleId(0)
    for accent_phrases in open_jtalk.analyze(TEXT):
        typecheck_dataclass(accent_phrases)
    typecheck_dataclass(synthesizer.create_audio_query(TEXT, STYLE_ID))


def test_user_dict_word() -> None:
    word = UserDictWord("単語", "タンゴ", 3)
    typecheck_dataclass(word)


@pytest.fixture
def onnxruntime() -> Onnxruntime:
    return Onnxruntime.load_once(filename=conftest.onnxruntime_filename)


@pytest.fixture
def open_jtalk() -> OpenJtalk:
    return OpenJtalk(conftest.open_jtalk_dic_dir)


@pytest.fixture
def model() -> VoiceModelFile:
    return VoiceModelFile.open(conftest.model_dir)


@pytest.fixture
def synthesizer(
    onnxruntime: Onnxruntime, open_jtalk: OpenJtalk, model: VoiceModelFile
) -> Synthesizer:
    synthesizer = Synthesizer(onnxruntime, open_jtalk, acceleration_mode="CPU")
    synthesizer.load_voice_model(model)
    return synthesizer


def typecheck_dataclass(instance: object) -> None:
    if not dataclasses.is_dataclass(instance):
        raise ValueError()
    for field in dataclasses.fields(instance):
        _typecheck_dataclass_field(getattr(instance, field.name), field.type)


def _typecheck_dataclass_field(field_value: object, field_type: object) -> None:
    if dataclasses.is_dataclass(field_type):
        assert isinstance(field_type, type) and isinstance(field_value, field_type)
        typecheck_dataclass(field_value)
    elif typing.get_origin(field_type) in (Union, UnionType):
        for tp in typing.get_args(field_type):
            try:
                _typecheck_dataclass_field(field_value, tp)
                break
            except AssertionError:
                pass
        else:
            assert False
    elif typing.get_origin(field_type) is list:
        assert isinstance(field_value, list)
        (element_type,) = typing.get_args(field_type)
        for element_value in field_value:
            _typecheck_dataclass_field(element_value, element_type)
    elif typing.get_origin(field_type) is Literal:
        assert field_value in typing.get_args(field_type)
    elif isinstance(field_type, NewType):
        _typecheck_dataclass_field(field_value, field_type.__supertype__)
    elif field_type is None:
        assert field_value is None
    else:
        assert isinstance(field_type, type) and isinstance(field_value, field_type)
