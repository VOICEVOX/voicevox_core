# pyright: strict

import ast
import dataclasses
import importlib
import textwrap
from ast import Assign, AsyncFor, AsyncFunctionDef, Await, ClassDef, FunctionDef, Module
from asyncio import AbstractEventLoop
from collections.abc import AsyncIterable, Awaitable, Iterator
from doctest import DocTest, DocTestParser, DocTestRunner, Example
from pathlib import Path
from typing import Literal, TypeAlias, TypeVar

import pytest
import voicevox_core
from voicevox_core import AudioQuery

Asyncness: TypeAlias = Literal["blocking", "asyncio"]


@dataclasses.dataclass
class Spec:
    examples: list[Example]
    asyncness: Asyncness
    module_name: str
    item_path: list[str]
    filename: str
    lineno: int
    docstring: str | None

    @property
    def name(self) -> str:
        return ".".join([self.module_name, *self.item_path])


def extract(pyi: Path, module_name: str, asyncness: Asyncness) -> Iterator[Spec]:
    parser = DocTestParser()

    def extract_from_node(
        node: AsyncFunctionDef | FunctionDef | ClassDef | Module,
        item_path: list[str],
    ) -> Iterator[Spec]:
        if docstring := ast.get_docstring(node, clean=False):
            if examples := parser.get_examples(docstring):
                if asyncness == "asyncio":
                    examples = [
                        Example(
                            replace_await(example.source),
                            example.want,
                            exc_msg=example.exc_msg,
                            lineno=example.lineno,
                            indent=example.indent,
                            options=example.options,
                        )
                        for example in examples
                    ]
                yield Spec(
                    examples,
                    asyncness,
                    module_name,
                    item_path,
                    str(pyi),
                    node.body[0].lineno - 1,
                    docstring,
                )
        for child in node.body:
            if isinstance(child, (AsyncFunctionDef, FunctionDef, ClassDef)):
                yield from extract_from_node(child, [*item_path, child.name])

    module = ast.parse(pyi.read_text(encoding="utf-8"))
    yield from extract_from_node(module, [])


def replace_await(source: str) -> str:
    module = ast.parse(source)
    if len(module.body) == 1:
        (stmt,) = module.body
        if isinstance(stmt, Assign):
            if len(stmt.targets) == 1:
                (target,) = stmt.targets
                if isinstance(stmt.value, Await):
                    return (
                        f"{ast.unparse(target)} = "
                        f"_run_awaitable({ast.unparse(stmt.value.value)})"
                    )
        elif isinstance(stmt, AsyncFor):
            if stmt.orelse:
                raise RuntimeError("unimplemented")
            return (
                f"for {ast.unparse(stmt.target)} in _run_aiter({ast.unparse(stmt.iter)}):\n"
                + "".join(
                    textwrap.indent(ast.unparse(stmt), "    ") + "\n"
                    for stmt in stmt.body
                )
            )
    if "async " in source or "await " in source:
        raise RuntimeError("unimplemented")
    return source


@pytest.mark.parametrize(
    "spec",
    [
        pytest.param(spec, id=spec.name)
        for specs in [
            extract(
                Path("./python/voicevox_core/_rust/asyncio.pyi"),
                "voicevox_core._rust.asyncio",
                "asyncio",
            ),
            extract(
                Path("./python/voicevox_core/_rust/blocking.pyi"),
                "voicevox_core._rust.blocking",
                "blocking",
            ),
        ]
        for spec in specs
    ],
)
def test(
    spec: Spec,
    blocking_synthesizesr_with_model: voicevox_core.blocking.Synthesizer,
    asyncio_synthesizesr_with_model: voicevox_core.asyncio.Synthesizer,
    event_loop: AbstractEventLoop,
) -> None:
    def do_something(_: object) -> None:
        pass

    globs: dict[str, object] = {
        "STYLE_ID": 302,
        "_do_something": do_something,
        "query": AudioQuery.from_accent_phrases([]),
        "voicevox_core": voicevox_core,
    }
    match spec.asyncness:
        case "blocking":
            globs["synth"] = blocking_synthesizesr_with_model
        case "asyncio":
            T = TypeVar("T")

            def run_awaitable(awaitable: Awaitable[T]) -> T:
                return event_loop.run_until_complete(awaitable)

            def run_aiter(xs: AsyncIterable[T]) -> Iterator[T]:
                it = aiter(xs)
                while True:
                    try:
                        yield run_awaitable(anext(it))
                    except StopAsyncIteration:
                        return

            globs["_run_aiter"] = run_aiter
            globs["_run_awaitable"] = run_awaitable
            globs["synth"] = asyncio_synthesizesr_with_model
    if spec.item_path:
        obj_name, *_ = spec.item_path
        globs[obj_name] = getattr(importlib.import_module(spec.module_name), obj_name)

    out: list[str] = []
    results = DocTestRunner().run(
        DocTest(
            spec.examples, globs, spec.name, spec.filename, spec.lineno, spec.docstring
        ),
        out=out.append,
    )
    if results.failed:
        pytest.fail("".join(out), pytrace=False)
