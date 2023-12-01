# Configuration file for the Sphinx documentation builder.
#
# For the full list of built-in configuration values, see the documentation:
# https://www.sphinx-doc.org/en/master/usage/configuration.html

# -- Project information -----------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#project-information

project = "verbs"
copyright = "2023, Simtopia"
author = "Simtopia"
release = "0.1.0"

# -- General configuration ---------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#general-configuration

extensions = []

templates_path = ["_templates"]
exclude_patterns = []


# -- Options for HTML output -------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#options-for-html-output

html_theme = "sphinx_material"
html_static_path = ["_static"]
html_title = "VERBS"
html_theme_options = {
    # Set the name of the project to appear in the navigation.
    "nav_title": "VERBS",
    # Set the color and the accent color
    "color_primary": "blue",
    "color_accent": "light-blue",
    # Set the repo location to get a badge with stats
    # 'repo_url': 'https://github.com/project/project/',
    # 'repo_name': 'Project',
    # Visible levels of the global TOC; -1 means unlimited
    "globaltoc_depth": 3,
    # If False, expand all TOC entries
    "globaltoc_collapse": False,
    # If True, show hidden TOC entries
    "globaltoc_includehidden": False,
}
