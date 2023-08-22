use ethers_contract::BaseContract;
use ethers_core::abi::{Detokenize, Tokenize};
use ethers_core::types::{Address as EthAddress, Bytes as EthersBytes, Selector, H256};
use revm::primitives::{Address, Bytecode, Bytes, Log, Output, TransactTo, TxEnv, U256};
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

pub struct CallResult {
    /// Flag whether transaction was successful.
    pub success: bool,
    /// Output data.
    pub output: Output,
    /// Vec of events returned by call.
    pub events: Vec<Event>,
}

/// Wraps event logs along with event information
pub struct Event {
    /// Name of the function being called.
    pub function_name: &'static str,
    /// Index of the contract being called.
    pub contract_idx: usize,
    /// Event data
    pub log: Log,
    /// Step event was created
    pub step: i64,
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
        let result = self.abi.encode(function_name, args);
        let encoded = match result {
            Ok(encoded) => encoded,
            Err(err) => panic!("Error encoding arguments to {}: {:?}", function_name, err),
        };

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
        let result = self.abi.encode_with_selector(selector, args);
        let encoded = match result {
            Ok(encoded) => encoded,
            Err(err) => panic!("Error encoding arguments: {:?}", err),
        };

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
        let result = self.abi.decode_output(function_name, output_data);
        match result {
            Ok(result) => result,
            Err(err) => panic!("Error decoding output from {}: {:?}", function_name, err),
        }
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
        let result = self.abi.decode_output_with_selector(selector, output_data);
        match result {
            Ok(result) => result,
            Err(err) => panic!("Error decoding output: {:?}", err),
        }
    }

    /// Decode contract event
    ///
    /// # Arguments
    ///
    /// * `event_name` - Name of the event.
    /// * `event` - Event struct
    ///
    pub fn decode_event<D: Detokenize>(&self, event_name: &'static str, event: Log) -> D {
        let topics: Vec<H256> = event
            .topics
            .into_iter()
            .map(|x| H256::from_slice(x.as_bytes()))
            .collect();
        let result = self
            .abi
            .decode_event(event_name, topics, EthersBytes(event.data));
        match result {
            Ok(result) => result,
            Err(err) => panic!("Error decoding event {}: {:?}", event_name, err),
        }
    }
}
