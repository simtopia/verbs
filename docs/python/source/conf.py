import pkg_resources

project = "VERBS"
copyright = "2023, Simtopia"
author = "Simtopia"
release = pkg_resources.get_distribution("verbs").version

extensions = [
    "sphinx.ext.napoleon",
    "sphinx.ext.autodoc",
    "sphinx.ext.autosummary",
    "sphinx.ext.intersphinx",
    "sphinx.ext.doctest",
    "sphinx.ext.coverage",
    "sphinx.ext.viewcode",
    "sphinx_copybutton",
]

napoleon_google_docstring = False
napoleon_numpy_docstring = True
napoleon_include_init_with_doc = False
napoleon_include_private_with_doc = False
napoleon_include_special_with_doc = False
napoleon_use_admonition_for_examples = False
napoleon_use_admonition_for_notes = True
napoleon_use_param = True
napoleon_use_ivar = False
napoleon_use_rtype = False
napoleon_preprocess_types = True
napoleon_attr_annotations = True

add_module_names = False

autodoc_class_signature = "separated"

templates_path = ["_templates"]
exclude_patterns = []

intersphinx_mapping = {
    "python": ("https://docs.python.org/3", None),
    "numpy": ("https://numpy.org/doc/stable", None),
    "pandas": ("https://pandas.pydata.org/docs/", None),
}

html_theme = "furo"
html_show_sourcelink = False
