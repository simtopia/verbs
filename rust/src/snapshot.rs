use alloy_primitives::{Address, Bytes, B256, U256};
use pyo3::{types::PyBytes, Python};
use revm::{
    db::{AccountState, CacheDB, DatabaseRef, DbAccount, EmptyDB},
    primitives::{AccountInfo, Bytecode, Log},
};
use rust_sim::network::Network;

use super::types::{address_to_py, bytes_to_py};

pub type PyAccountInfo<'a> = (&'a PyBytes, u64, &'a PyBytes, Option<&'a PyBytes>);

pub type PyDbAccount<'a> = (PyAccountInfo<'a>, u8, Vec<(&'a PyBytes, &'a PyBytes)>);

pub type PyLog<'a> = (&'a PyBytes, Vec<&'a PyBytes>, &'a PyBytes);

pub type PyDbState<'a> = (
    // Admin Account Address
    String,
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
        _ => panic!("Got an unexpected value to cast to an account state"),
    }
}

pub fn create_py_snapshot<'a, D: DatabaseRef>(
    py: Python<'a>,
    network: &mut Network<D>,
) -> PyDbState<'a> {
    let db = network.evm.db().unwrap();

    let admin_address = network.admin_address.to_string();

    let accounts: Vec<(&'a PyBytes, PyDbAccount<'a>)> = db
        .accounts
        .iter()
        .map(|(k, v)| {
            (
                address_to_py(py, *k),
                (
                    (
                        PyBytes::new(py, v.info.balance.as_le_slice()),
                        v.info.nonce,
                        PyBytes::new(py, v.info.code_hash.as_slice()),
                        v.info
                            .code
                            .as_ref()
                            .map(|b| bytes_to_py(py, b.bytes().clone())),
                    ),
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
        .contracts
        .iter()
        .map(|(k, v)| {
            (
                PyBytes::new(py, k.as_slice()),
                bytes_to_py(py, v.bytecode.clone()),
            )
        })
        .collect();

    let logs: Vec<PyLog> = db
        .logs
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
        .block_hashes
        .iter()
        .map(|(k, v)| {
            (
                PyBytes::new(py, k.as_le_slice()),
                PyBytes::new(py, v.as_slice()),
            )
        })
        .collect();

    (admin_address, accounts, contracts, logs, block_hashes)
}

pub fn load_snapshot(db: &mut CacheDB<EmptyDB>, snapshot: PyDbState) {
    for (k, v) in snapshot.1.into_iter() {
        db.accounts.insert(
            Address::from_slice(k.as_bytes()),
            DbAccount {
                info: AccountInfo {
                    balance: U256::from_le_slice(v.0 .0.as_bytes()),
                    nonce: v.0 .1,
                    code_hash: B256::from_slice(v.0 .2.as_bytes()),
                    code: v
                        .0
                         .3
                        .map(|x| Bytecode::new_raw(Bytes::copy_from_slice(x.as_bytes()))),
                },
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
