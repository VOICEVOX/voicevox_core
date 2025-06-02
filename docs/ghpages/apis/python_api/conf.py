# Configuration file for the Sphinx documentation builder.
#
# For the full list of built-in configuration values, see the documentation:
# https://www.sphinx-doc.org/en/master/usage/configuration.html

# -- Project information -----------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#project-information

import typing

import sphinx.util.logging
from docutils.nodes import TextElement, reference
from sphinx.addnodes import pending_xref
from sphinx.application import Sphinx
from sphinx.environment import BuildEnvironment

project = "voicevox_core_python_api"
# copyright = '2022, _'
# author = '_'
# release = '_'

# -- General configuration ---------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#general-configuration

autodoc_docstring_signature = True

extensions = ["autoapi.extension", "sphinx.ext.napoleon"]

autoapi_type = "python"
autoapi_dirs = ["../../../../crates/voicevox_core_python_api/python"]
autoapi_file_patterns = ["*.pyi", "*.py"]
autoapi_ignore = ["*test*"]
autoapi_options = [
    "members",
    "undoc-members",
    "show-inheritance",
    "show-module-summary",
    "special-members",
    "imported-members",
]

# templates_path = ['_templates']
exclude_patterns = ["autoapi/*/_rust/*"] # パブリックAPIを意図した部分ではなく、またorphan扱いとなって警告が出るため


# -- Options for HTML output -------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#options-for-html-output

html_theme = "pydata_sphinx_theme"
# html_static_path = ['_static']


def setup(sphinx: Sphinx) -> None:
    sphinx.connect("missing-reference", _on_missing_reference)


def _on_missing_reference(
    app: Sphinx, env: BuildEnvironment, node: pending_xref, contnode: TextElement
) -> reference | None:
    """
    ``NewType`` や ``TypeAlias`` について ``class``
    宛てにリンクしようとしてmissingになったものを、 ``data`` 宛てに修正する。
    """
    # 参考: https://github.com/sphinx-doc/sphinx/issues/10785#issue-1348601826
    TARGETS = {
        "AccelerationMode",
        "CharacterVersion",
        "StyleId",
        "StyleType",
        "UserDictWordType",
        "VoiceModelId",
    }
    if (
        node["refdomain"] == "py"
        and node["reftype"] == "class"
        and node["reftarget"].split(".")[-1] in TARGETS
    ):
        xref = app.env.get_domain("py").resolve_xref(
            env, node["refdoc"], app.builder, "data", node["reftarget"], node, contnode
        )
        xref = typing.cast(reference | None, xref)  # ?
        if not xref:
            _logger.error("unresolved link to `%s`", node["reftarget"])
        return xref


_logger = sphinx.util.logging.getLogger(__name__)
