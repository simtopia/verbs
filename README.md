# VERBS

Python ABM library built around Rust backend.

## Getting Started

### Building from Source

Building VERBS requires [maturin to be installed](https://www.maturin.rs/installation).

> :warning: On OSX `patchelf`` should be manually installed using
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

VERBS can be added as a git dependency to your projects `pyproject.toml`
but requires maturin to used as a build backend, for example

```
[build-system]
requires = ["maturin>=1.2,<2.0"]
build-backend = "maturin"
```

## Examples

Examples of models implemented using VERBS can be
found in `/examples`.

## Developers & Contributing

see [here](.github/docs/developers.md) for developer notes.

## Rust Package

The core rust simulation engine can be used to write simulation
purely in Rust, with a significant gain in performance over
Python in most cases. See [here](.github/docs/rust.md).
