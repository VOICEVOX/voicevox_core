from typing import Any, NoReturn

from pydantic import GetCoreSchemaHandler
from pydantic_core import CoreSchema, core_schema

__all__ = ["_Reserved"]


class _Reserved(str):
    def __new__(cls) -> NoReturn:
        raise TypeError()

    @classmethod
    def __get_pydantic_core_schema__(
        cls, source_type: Any, handler: GetCoreSchemaHandler
    ) -> CoreSchema:
        _ = source_type, handler
        # TODO: pydantic/pydantic-core#1579 がリリースに入ったら`NeverSchema`にする
        return core_schema.no_info_after_validator_function(
            cls._no_input_allowed, core_schema.any_schema()
        )

    @classmethod
    def _no_input_allowed(cls, _: object) -> NoReturn:
        raise ValueError(f"No input is allowed for `{cls.__name__}`")
