# VERBS

Python ABM library built around Rust backend.

Full documentation can be found [here](https://simtopia.github.io/verbs/).

## Getting Started

### Installation

VERBS can be installed via pip using

```
pip install verbs
```

### Building from Source

Building VERBS requires [maturin to be installed](https://www.maturin.rs/installation).

> :warning: On OSX ``patchelf`` should be manually installed using
  [homebrew](https://brew.sh)

The Python package can be built using [hatch](https://hatch.pypa.io/latest/)
by running

```
hatch run dev:build
```

### Jupyter Notebook

A jupyter notebook with VERBS installed as a dependency can be
run using [hatch](https://hatch.pypa.io/latest/)

```
hatch run notebook:jupyter
```

### Git Dependency

VERBS can be added as a direct dependency to your projects `pyproject.toml`
but requires maturin to be added as a build requirement, for example

```
[build-system]
requires = ["setuptools >= 61.0", "maturin>=1.2,<2.0"]
build-backend = "setuptools.build_meta"
```

## Examples

Examples of models implemented using VERBS can be
found in `/examples`. Larger examples can also be found in this
[repo](https://github.com/simtopia/verbs-examples).

## Developers & Contributing

VERBS is under active development, if you notice a problem
or have a suggestion please [open an issue](https://github.com/simtopia/verbs/issues).

We welcome contributions to this project, see [here](https://github.com/simtopia/verbs/blob/main/.github/docs/developers.md)
for developer notes.

## Rust Package

The core rust simulation engine can be used to write simulation
purely in Rust, with a significant gain in performance over
Python in most cases. See [here](https://github.com/simtopia/verbs/blob/main/.github/docs/rust.md)
for usage notes.
