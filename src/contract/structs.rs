use ethers_contract::BaseContract;
use ethers_core::abi::{Detokenize, Tokenize};
use ethers_core::types::Selector;
use ethers_core::types::{Address as EthAddress, Bytes as EthersBytes};
use revm::primitives::{Address, Bytecode, Bytes, U256};
use revm::primitives::{TransactTo, TxEnv};
use std::collections::HashMap;

pub struct ContractDefinition {
    pub name: String,
    pub abi: BaseContract,
    pub bytecode: Bytecode,
    pub arguments: Bytes,
    pub deploy_address: Address,
    pub storage_values: Option<HashMap<U256, U256>>,
}

pub struct DeployedContract {
    pub name: String,
    pub abi: BaseContract,
    pub address: Address,
    pub arg_address: EthAddress,
}

pub struct Transaction<T: Tokenize> {
    pub callee: Address,
    pub contract_idx: usize,
    pub function_name: &'static str,
    pub args: T,
}

pub struct Call {
    pub function_name: &'static str,
    pub contract_idx: usize,
    pub callee: Address,
    pub transact_to: Address,
    pub args: EthersBytes,
}

impl DeployedContract {
    pub fn encode_transaction<T: Tokenize>(&self, transaction: Transaction<T>) -> Call {
        let encoded_args = self
            .abi
            .encode(transaction.function_name, transaction.args)
            .unwrap();

        Call {
            function_name: transaction.function_name,
            contract_idx: transaction.contract_idx,
            callee: transaction.callee,
            transact_to: self.address,
            args: encoded_args,
        }
    }

    pub fn unwrap_transaction<'a, T: Tokenize>(
        &self,
        callee: Address,
        function_name: &'static str,
        args: T,
    ) -> TxEnv {
        let encoded = self.abi.encode(function_name, args).unwrap();

        TxEnv {
            caller: callee,
            gas_limit: u64::MAX,
            gas_price: U256::ZERO,
            gas_priority_fee: None,
            transact_to: TransactTo::Call(self.address),
            value: U256::ZERO,
            data: encoded.0,
            chain_id: None,
            nonce: None,
            access_list: Vec::new(),
        }
    }

    pub fn unwrap_transaction_with_selector<'a, T: Tokenize>(
        &self,
        callee: Address,
        selector: Selector,
        args: T,
    ) -> TxEnv {
        let encoded = self.abi.encode_with_selector(selector, args).unwrap();

        TxEnv {
            caller: callee,
            gas_limit: u64::MAX,
            gas_price: U256::ZERO,
            gas_priority_fee: None,
            transact_to: TransactTo::Call(self.address),
            value: U256::ZERO,
            data: encoded.0,
            chain_id: None,
            nonce: None,
            access_list: Vec::new(),
        }
    }

    pub fn unwrap_call(call: Call) -> TxEnv {
        TxEnv {
            caller: call.callee,
            gas_limit: u64::MAX,
            gas_price: U256::ZERO,
            gas_priority_fee: None,
            transact_to: TransactTo::Call(call.transact_to),
            value: U256::ZERO,
            data: call.args.0,
            chain_id: None,
            nonce: None,
            access_list: Vec::new(),
        }
    }

    pub fn decode_output<D: Detokenize>(
        &self,
        function_name: &'static str,
        output_data: bytes::Bytes,
    ) -> D {
        self.abi.decode_output(function_name, output_data).unwrap()
    }

    pub fn decode_output_with_selector<D: Detokenize>(
        &self,
        selector: Selector,
        output_data: bytes::Bytes,
    ) -> D {
        self.abi
            .decode_output_with_selector(selector, output_data)
            .unwrap()
    }
}
