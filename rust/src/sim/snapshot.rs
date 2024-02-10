use alloy_primitives::{Address, Bytes, B256, U256};
use db::{LocalDB, Requests, DB};
use pyo3::{types::PyBytes, Python};
use revm::{
    db::{AccountState, DbAccount},
    primitives::{AccountInfo, BlobExcessGasAndPrice, BlockEnv, Bytecode, Log},
};
use rust_sim::network::Network;

use crate::types::{address_to_py, bytes_to_py};

pub type PyBlockEnv<'a> = (
    &'a PyBytes,
    &'a PyBytes,
    &'a PyBytes,
    &'a PyBytes,
    &'a PyBytes,
    &'a PyBytes,
    Option<&'a PyBytes>,
    Option<(u64, u128)>,
);

pub type PyAccountInfo<'a> = (&'a PyBytes, u64, &'a PyBytes, Option<&'a PyBytes>);

pub type PyDbAccount<'a> = (PyAccountInfo<'a>, u8, Vec<(&'a PyBytes, &'a PyBytes)>);

fn info_to_py<'a>(py: Python<'a>, info: &AccountInfo) -> PyAccountInfo<'a> {
    (
        PyBytes::new(py, info.balance.as_le_slice()),
        info.nonce,
        PyBytes::new(py, info.code_hash.as_slice()),
        info.code
            .as_ref()
            .map(|b| bytes_to_py(py, b.bytes().clone())),
    )
}

fn py_to_info(info: PyAccountInfo) -> AccountInfo {
    AccountInfo {
        balance: U256::from_le_slice(info.0.as_bytes()),
        nonce: info.1,
        code_hash: B256::from_slice(info.2.as_bytes()),
        code: info
            .3
            .map(|x| Bytecode::new_raw(Bytes::copy_from_slice(x.as_bytes()))),
    }
}

pub type PyLog<'a> = (&'a PyBytes, Vec<&'a PyBytes>, &'a PyBytes);

pub type PyDbState<'a> = (
    // EVM Block Env
    PyBlockEnv<'a>,
    // Accounts
    Vec<(&'a PyBytes, PyDbAccount<'a>)>,
    // Contracts
    Vec<(&'a PyBytes, &'a PyBytes)>,
    // Logs
    Vec<PyLog<'a>>,
    // Block hashes
    Vec<(&'a PyBytes, &'a PyBytes)>,
);

fn account_state_to_int(account_state: &AccountState) -> u8 {
    match account_state {
        AccountState::NotExisting => 0,
        AccountState::Touched => 1,
        AccountState::StorageCleared => 2,
        AccountState::None => 3,
    }
}

fn int_to_account_state(i: u8) -> AccountState {
    match i {
        0 => AccountState::NotExisting,
        1 => AccountState::Touched,
        2 => AccountState::StorageCleared,
        3 => AccountState::None,
        _ => panic!("Got an unexpected value {} to cast to an account state", i),
    }
}

pub type PyRequests<'a> = (
    u128,
    u128,
    Vec<(&'a PyBytes, PyAccountInfo<'a>)>,
    Vec<(&'a PyBytes, &'a PyBytes, &'a PyBytes)>,
);

pub fn create_py_request_history<'a>(py: Python<'a>, requests: &Requests) -> PyRequests<'a> {
    let timestamp: u128 = requests.start_timestamp.try_into().unwrap();
    let block_number: u128 = requests.start_block_number.try_into().unwrap();

    let py_accounts: Vec<(&'a PyBytes, PyAccountInfo)> = requests
        .accounts
        .iter()
        .map(|(address, info)| (address_to_py(py, *address), info_to_py(py, info)))
        .collect();

    let py_storage: Vec<(&'a PyBytes, &'a PyBytes, &'a PyBytes)> = requests
        .storage
        .iter()
        .map(|(address, idx, value)| {
            (
                address_to_py(py, *address),
                PyBytes::new(py, idx.as_le_slice()),
                PyBytes::new(py, value.as_le_slice()),
            )
        })
        .collect();

    (timestamp, block_number, py_accounts, py_storage)
}

pub fn create_py_snapshot<'a, D: DB>(py: Python<'a>, network: &mut Network<D>) -> PyDbState<'a> {
    let block = network.evm.env.block.clone();

    let block_env = (
        PyBytes::new(py, block.number.as_le_slice()),
        address_to_py(py, block.coinbase),
        PyBytes::new(py, block.timestamp.as_le_slice()),
        PyBytes::new(py, block.gas_limit.as_le_slice()),
        PyBytes::new(py, block.basefee.as_le_slice()),
        PyBytes::new(py, block.difficulty.as_le_slice()),
        block.prevrandao.map(|x| PyBytes::new(py, x.as_slice())),
        block
            .blob_excess_gas_and_price
            .map(|x| (x.excess_blob_gas, x.blob_gasprice)),
    );

    let db = network.evm.db().unwrap();

    let accounts: Vec<(&'a PyBytes, PyDbAccount<'a>)> = db
        .accounts()
        .iter()
        .map(|(k, v)| {
            (
                address_to_py(py, *k),
                (
                    info_to_py(py, &v.info),
                    account_state_to_int(&v.account_state),
                    v.storage
                        .iter()
                        .map(|(k, v)| {
                            (
                                PyBytes::new(py, k.as_le_slice()),
                                PyBytes::new(py, v.as_le_slice()),
                            )
                        })
                        .collect::<Vec<(&'a PyBytes, &'a PyBytes)>>(),
                ),
            )
        })
        .collect();

    let contracts: Vec<(&'a PyBytes, &'a PyBytes)> = db
        .contracts()
        .iter()
        .map(|(k, v)| {
            (
                PyBytes::new(py, k.as_slice()),
                bytes_to_py(py, v.bytecode.clone()),
            )
        })
        .collect();

    let logs: Vec<PyLog> = db
        .logs()
        .iter()
        .map(|x| {
            (
                address_to_py(py, x.address),
                x.topics
                    .iter()
                    .map(|x| PyBytes::new(py, x.as_slice()))
                    .collect(),
                bytes_to_py(py, x.data.clone()),
            )
        })
        .collect();

    let block_hashes: Vec<(&'a PyBytes, &'a PyBytes)> = db
        .block_hashes()
        .iter()
        .map(|(k, v)| {
            (
                PyBytes::new(py, k.as_le_slice()),
                PyBytes::new(py, v.as_slice()),
            )
        })
        .collect();

    (block_env, accounts, contracts, logs, block_hashes)
}

pub fn load_cache(cache: PyRequests, db: &mut LocalDB) {
    for (address, info) in cache.2.into_iter() {
        db.accounts.insert(
            Address::from_slice(address.as_bytes()),
            DbAccount {
                info: py_to_info(info),
                ..Default::default()
            },
        );
    }

    for (address, idx, value) in cache.3.into_iter() {
        let address = Address::from_slice(address.as_bytes());
        db.accounts.get_mut(&address).unwrap().storage.insert(
            U256::from_le_slice(idx.as_bytes()),
            U256::from_le_slice(value.as_bytes()),
        );
    }
}

pub fn load_block_env(snapshot: &PyDbState) -> BlockEnv {
    let block = snapshot.0;

    BlockEnv {
        number: U256::from_le_slice(block.0.as_bytes()),
        coinbase: Address::from_slice(block.1.as_bytes()),
        timestamp: U256::from_le_slice(block.2.as_bytes()),
        gas_limit: U256::from_le_slice(block.3.as_bytes()),
        basefee: U256::from_le_slice(block.4.as_bytes()),
        difficulty: U256::from_le_slice(block.5.as_bytes()),
        prevrandao: block.6.map(|x| B256::from_slice(x.as_bytes())),
        blob_excess_gas_and_price: block.7.map(|x| BlobExcessGasAndPrice {
            excess_blob_gas: x.0,
            blob_gasprice: x.1,
        }),
    }
}

pub fn load_snapshot(db: &mut LocalDB, snapshot: PyDbState) {
    for (k, v) in snapshot.1.into_iter() {
        db.accounts.insert(
            Address::from_slice(k.as_bytes()),
            DbAccount {
                info: py_to_info(v.0),
                account_state: int_to_account_state(v.1),
                storage: v
                    .2
                    .into_iter()
                    .map(|(a, b)| {
                        (
                            U256::from_le_slice(a.as_bytes()),
                            U256::from_le_slice(b.as_bytes()),
                        )
                    })
                    .collect(),
            },
        );
    }

    for (k, v) in snapshot.2.into_iter() {
        db.contracts.insert(
            B256::from_slice(k.as_bytes()),
            Bytecode::new_raw(Bytes::copy_from_slice(v.as_bytes())),
        );
    }

    for log in snapshot.3.into_iter() {
        db.logs.push(Log {
            address: Address::from_slice(log.0.as_bytes()),
            topics: log
                .1
                .into_iter()
                .map(|x| B256::from_slice(x.as_bytes()))
                .collect(),
            data: Bytes::copy_from_slice(log.2.as_bytes()),
        })
    }

    for (k, v) in snapshot.4.into_iter() {
        db.block_hashes.insert(
            U256::from_le_slice(k.as_bytes()),
            B256::from_slice(v.as_bytes()),
        );
    }
}
