from typing import NoReturn

__all__ = ["_Reserved"]


class _Reserved(str):
    def __new__(cls) -> NoReturn:
        raise TypeError()
