import pydantic
import pytest
from pydantic import TypeAdapter
from voicevox_core import AccelerationMode, StyleType, UserDictWordType


def test_invalid_input() -> None:
    for ty in [AccelerationMode, StyleType, UserDictWordType]:
        with pytest.raises(pydantic.ValidationError, match="^2 validation errors for"):
            TypeAdapter(ty).validate_python("不正な文字列")
