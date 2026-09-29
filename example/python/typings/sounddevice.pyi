from types import TracebackType
from typing import Literal

class RawOutputStream:
    def __init__(
        self,
        samplerate: float | None = None,
        blocksize: None = None,  # ?
        device: None = None,  # ?
        channels: int | None = None,
        dtype: Literal["float32", "int32", "int24", "int16", "int8", "uint8"]
        | None = None,
        latency: float | Literal["low", "high"] | None = None,
        extra_settings: None = None,  # ?
        callback: None = None,  # ?
        finished_callback: None = None,  # ?
        clip_off: None = None,  # ?
        dither_off: None = None,  # ?
        never_drop_input: None = None,  # ?
        prime_output_buffers_using_stream_callback: None = None,  # ?
    ) -> None: ...
    def __enter__(self) -> "RawOutputStream": ...
    def __exit__(
        self,
        exc_type: type[BaseException] | None,
        exc_value: BaseException | None,
        traceback: TracebackType | None,
    ) -> None: ...
    def write(self, data: bytes) -> bool: ...
