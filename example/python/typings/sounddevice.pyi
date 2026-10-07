from types import TracebackType
from typing import Literal, NoReturn, final

class RawOutputStream:
    def __init__(
        self,
        *,
        samplerate: float | _Unknown,
        channels: int | _Unknown,
        dtype: Literal["int16"] | _Unknown,
        latency: float | _Unknown,
    ) -> None: ...
    def __enter__(self) -> RawOutputStream: ...
    def __exit__(
        self,
        exc_type: type[BaseException] | None,
        exc_value: BaseException | None,
        traceback: TracebackType | None,
    ) -> None: ...
    def write(self, data: bytes | _Unknown) -> bool: ...

@final
class _Unknown:
    def __new__(cls) -> NoReturn: ...
