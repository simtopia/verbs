# Rust Ethereum ABM

Built around [Rust Ethereum Virtual Machine](https://github.com/bluealloy/revm)

## Examples

Examples can be run using cargo.

- Basic sim demonstrating agents moving around an ERC20 token

  ```
  cargo run --example basic_sim <N-steps> <N-agents>
  ```

- Initialising EVM state from mainet

  ```
  cargo run --release --example forked_sim <ALCHEMY-API-KEY>
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
- Bytecode can be taken from live deployments or compiled from
  a solidity using the Python script `scripts/process_contract.py`

  ```bash
  python scripts/process_contract.py <PATH_TO_CONTRACT> <SOL_VERSION> <OUTPUT_FOLDER>
  ```
