use ethers_contract::BaseContract;
use ethers_core::abi::{Detokenize, Tokenize};
use ethers_core::types::Selector;
use ethers_core::types::{Address as EthAddress, Bytes as EthersBytes};
use revm::primitives::{Address, Bytecode, Bytes, U256};
use revm::primitives::{TransactTo, TxEnv};
use std::collections::HashMap;

/// Collection of data used to deploy a new contract.
pub struct ContractDefinition {
    /// Name of the contract.
    pub name: String,
    /// Contract ABI object.
    pub abi: BaseContract,
    /// Contract Bytecode.
    pub bytecode: Bytecode,
    /// Constructor arguments packed into bytes.
    pub arguments: Bytes,
    /// Desired deployment address of this contract.
    pub deploy_address: Address,
    /// Map of key-value storage pairs.
    pub storage_values: Option<HashMap<U256, U256>>,
}

/// Deployed contract
pub struct DeployedContract {
    /// Name of the contract.
    pub name: String,
    /// ABI contract object.
    pub abi: BaseContract,
    /// Revm address of the contract.
    pub address: Address,
    /// Ethers-core address of the contract.
    pub arg_address: EthAddress,
}

/// EVM call/transaction
pub struct Call {
    /// Name of the function being called.
    pub function_name: &'static str,
    /// Index of the contract being called.
    pub contract_idx: usize,
    /// Address of the contract caller
    pub callee: Address,
    /// Address to transact to
    pub transact_to: Address,
    /// Contract arguments represented as bytes
    pub args: EthersBytes,
    /// Flag if `true` the simulation will halt (panic)
    /// if this transaction fails.
    pub checked: bool,
}

/// Functionality attached to a deployed contract.
impl DeployedContract {
    /// Encode details of an EVM call into a transaction object to be
    /// processed by the EVM.
    ///
    /// # Arguments
    ///
    /// * `callee` - Address of the contract caller.
    /// * `function_name` - Name of the function being called.
    /// * `args` - Function arguments (a tuple if multiple).
    ///
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

    /// Encode details of an EVM call into a transaction using a
    /// selector object to select the function to be called. Used
    /// in the case multiple functions have the same name in a contract.
    ///
    /// # Arguments
    ///
    /// * `callee` - Address of the contract caller.
    /// * `selector` - Selector encoding the id of the function to call.
    /// * `args` - Function arguments (a tuple if multiple).
    ///
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

    /// Unwrap a call object into an EVM transaction object.
    ///
    /// # Arguments
    ///
    /// * `call` - Call data.
    ///
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

    /// Decode data returned from an EVM call into specific type
    ///
    /// # Arguments
    ///
    /// * `function_name` - Name of the function to call.
    /// * `output_data` - Bytes to decode back into data.
    ///
    pub fn decode_output<D: Detokenize>(
        &self,
        function_name: &'static str,
        output_data: bytes::Bytes,
    ) -> D {
        self.abi.decode_output(function_name, output_data).unwrap()
    }

    /// Decode output using a function selector.
    ///
    /// # Arguments
    ///
    /// * `selector` - Function selector object.
    /// * `output_data` - Bytes to decode back into data.
    ///
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
