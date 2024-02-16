# Developers

## Dependencies

This project uses [hatch](https://hatch.pypa.io/latest/) for
Python dependency management. Development tasks can also be run using hatch commands:

* Linting: `hatch run dev:lint`
* Tests: `hatch run dev:test`
* Examples: `hatch run dev:examples`
* Build: `hatch run dev:build`

## Structure

* `crates/`: Rust simulation environment crates, can also be
  used as an independent Rust library for simulation development.
* `docs/`: Sphinx documentations files
* `examples/`: Model examples.
* `rust/`: Rust-Python interface to the core simulation engine.
* `src/`: VERBS Python package.
* `tests/`: Python module tests

## Documentation

Python documentation can be built using [sphinx](https://www.sphinx-doc.org/en/master/)
by running

```
hatch run sphinx:build
```
