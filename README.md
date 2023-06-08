# Rust Ethereum ABM

Built around [Rust Ethereum Virtual Machine](https://github.com/bluealloy/revm)

## Examples

Examples can be run using cargo. For example:

```
cargo run --example basic_sim 200
```

## Loading Contracts

Contracts need to be in a specific format to be loaded for the simulation. This can
be done using the python script `scripts/process_contract.py`

```
python scripts/process_contract.py PATH_TO_CONTRACT SOL_VERSION OUTPUT_FOLDER
```
