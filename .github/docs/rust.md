# Rust

VERBS acts a Python interface to a rust evm (the
[revm](https://github.com/bluealloy/revm) project)
and rust simulation engine. This engine can be used
to write rust simulations purely in Python for a general
gain in performance over native Python.

## Structure

The core Rust library can be found in the `crates/` folder.
It currently consists of 3 sub-crates

* `crates/verbs_rs/`: Rust Ethereum ABM library.
* `crates/verbs_macros/`: Macros used by the `verbs-rs crate`.
* `crates/verbs_db`: Streamlined single threaded EVM database
  backend that allows for both in-memory and forking
  from deployed chains.

Cargo commands can be run from the repo root.

## Rust Examples

Examples of the libraries use can be found in
[crates/verbs_rs/examples](../../crates/verbs_rs/examples).

The examples can be run using cargo:

- Basic sim demonstrating agents moving around an ERC20 token

  ```
  cargo run --release --example basic_sim -- -s <N-steps> -n <N-agents>
  ```

- Initialising EVM state from mainet

  ```
  cargo run --release --example forked_sim -- -key <ALCHEMY-API-KEY>
  ```

## Loading Contracts

Contracts deployment requires the ABI and deployment bytecode:

- An ABI rust representation can generated using the `sol!` macro, e.g.

  ```rust
  use alloy_sol_types::sol;

  // From and abi string
  sol!(
    SomeABI,
    r#"<ABI-JSON-STRING>"#
  )

  // Or from an ABI file
  sol!(
    SomeABI,
    <PATH-TO-ABI-FILE>
  )
  ```
