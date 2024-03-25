//! Simulation environments
//!
//! A simulation environment wraps a local
//! EVM & database with tracking of simulation
//! events. It also provides functionality to
//! deploy contracts, interact contracts and
//! process queues of transactions.
//!

mod utils;
mod validator;

use crate::contract::{Event, Transaction};
use crate::utils::Eth;
use crate::{ForkDb, LocalDB, RequestCache, DB};
use alloy_primitives::{Address, FixedBytes, B256, U256};
use alloy_sol_types::SolCall;
use log::debug;
use rand::Rng;
use revm::primitives::{AccountInfo, Bytecode, ExecutionResult, Log, ResultAndState, TxEnv};
use revm::{ContextWithHandlerCfg, Evm, Handler};
pub use utils::{decode_event, process_events, RevertError};
pub use validator::{GasPriorityValidator, RandomValidator, Validator};

/// Simulation environment
///
/// Environment wrapping an in-memory EVM and
/// functionality to update the state of the
/// environment.
pub struct Env<D: DB, V: Validator> {
    /// Local EVM state
    pub evm_state: Option<ContextWithHandlerCfg<(), D>>,
    /// Events/updates in the last block
    pub last_events: Vec<Event>,
    /// History of events/updates over the
    /// lifetime of the environment
    pub event_history: Vec<Event>,
    /// Validator responsbile for transaction ordering
    pub validator: V,
}

/// EVM update methods
trait CallEVM {
    /// Execute a transaction, and update the EVM state
    fn execute(&mut self, tx: TxEnv) -> ExecutionResult;
    /// Execute a transaction without updating the EVM
    fn call(&mut self, tx: TxEnv) -> ResultAndState;
}

impl<'a, D: DB> CallEVM for Evm<'a, (), D> {
    fn execute(&mut self, tx: TxEnv) -> ExecutionResult {
        self.context.evm.env.tx = tx;

        match self.transact_commit() {
            Ok(val) => val,
            Err(e) => match e {
                revm::primitives::EVMError::Transaction(t) => {
                    panic!("Call failed: Invalid transaction {:?}", t)
                }
                revm::primitives::EVMError::Header(h) => {
                    panic!("Call failed: Invalid header {:?}", h)
                }
                revm::primitives::EVMError::Database(d) => {
                    panic!("Call failed: Database error {:?}", d)
                }
                revm::primitives::EVMError::Custom(c) => {
                    panic!("Call failed: Custom error {:?}", c)
                }
            },
        }
    }

    fn call(&mut self, tx: TxEnv) -> ResultAndState {
        self.context.evm.env.tx = tx;

        match self.transact() {
            Ok(val) => val,
            Err(e) => match e {
                revm::primitives::EVMError::Transaction(t) => {
                    panic!("Call failed: Invalid transaction {:?}", t)
                }
                revm::primitives::EVMError::Header(h) => {
                    panic!("Call failed: Invalid header {:?}", h)
                }
                revm::primitives::EVMError::Database(d) => {
                    panic!("Call failed: Database error {:?}", d)
                }
                revm::primitives::EVMError::Custom(c) => {
                    panic!("Call failed: Custom error {:?}", c)
                }
            },
        }
    }
}

impl<V: Validator> Env<ForkDb, V> {
    /// Initialise an environment with a forked DB
    ///
    /// Initialise a simulation environment with
    /// a database that can request values from
    /// a remote fork of the blockchain. During EVM
    /// execution, if a contract or storage value
    /// does not exist locally the db will attempt
    /// to request it from the remote endpoint.
    ///
    /// # Arguments
    ///
    /// * `node_url` - Url of service to make db requests
    /// * `block_number` - Block number to fork from, if None
    ///    latest available block will be used.
    ///
    pub fn init(node_url: &str, block_number: Option<u64>, validator: V) -> Self {
        let db = ForkDb::new(node_url, block_number);
        let timestamp = db.block.timestamp.as_u128();
        let block_number = match db.block.number {
            Some(n) => U256::try_from(n.as_u64()).unwrap(),
            None => U256::ZERO,
        };

        let evm = Evm::builder()
            .with_db(db)
            .modify_cfg_env(|cfg| {
                cfg.limit_contract_code_size = Some(0x1000000);
                cfg.disable_eip3607 = true;
            })
            .modify_block_env(|block| {
                block.gas_limit = U256::MAX;
                block.timestamp = U256::try_from(timestamp).unwrap();
                block.number = block_number;
            })
            .build();

        let context = evm.into_context_with_handler_cfg();

        Self {
            evm_state: Some(context),
            last_events: Vec::new(),
            event_history: Vec::new(),
            validator,
        }
    }

    /// Get history of data requests made by the DB
    ///
    /// Get a history of requests for data from the
    /// remote fork. These requests can then be inserted
    /// into an empty DB to speed up future simulations.
    ///
    pub fn get_request_history(&self) -> &RequestCache {
        match &self.evm_state {
            Some(e) => &e.context.evm.db.requests,
            None => panic!("No EVM state set"),
        }
    }
}

impl<V: Validator> Env<LocalDB, V> {
    /// Initialise a simulation with an in-memory DB
    ///
    /// Initialises a simulation environment with an
    /// empty in-memory database.
    ///
    /// # Arguments
    ///
    /// - `timestamp` - Timestamp to initialise the
    ///   the simulation/EVM with
    /// - `block_number` - Block number initialise the
    ///   the simulation/EVM with
    ///
    pub fn init(timestamp: U256, block_number: U256, validator: V) -> Self {
        let evm = Evm::builder()
            .with_db(LocalDB::new())
            .modify_cfg_env(|cfg| {
                cfg.limit_contract_code_size = Some(0x1000000);
                cfg.disable_eip3607 = true;
            })
            .modify_block_env(|block| {
                block.gas_limit = U256::MAX;
                block.timestamp = timestamp;
                block.number = block_number;
            })
            .build();

        let start_balance = U256::to_weth(10_000);

        let mut env = Self {
            evm_state: Some(evm.into_context_with_handler_cfg()),
            last_events: Vec::new(),
            event_history: Vec::new(),
            validator,
        };

        env.insert_account(Address::ZERO, start_balance);

        env
    }
}

impl<D: DB, V: Validator> Env<D, V> {
    fn evm(&mut self) -> Evm<(), D> {
        let state = self.evm_state.take();

        match state {
            Some(s) => {
                let ContextWithHandlerCfg { context, cfg } = s;
                Evm {
                    context,
                    handler: Handler::new(cfg),
                }
            }
            None => panic!("No EVM state set (this should not happen!)"),
        }
    }

    /// Get a mutable reference to the stored evm-state
    pub fn evm_state(&mut self) -> &mut ContextWithHandlerCfg<(), D> {
        match &mut self.evm_state {
            Some(e) => e,
            None => panic!("No EVM state set (this should not happen!)"),
        }
    }

    /// Increment block number, time and prevarando
    pub fn increment_time<R: Rng>(&mut self, rng: &mut R, interval: u64) {
        let state = self.evm_state();
        state.context.evm.env.block.timestamp += U256::from(interval);
        state.context.evm.env.block.number += U256::from(1);
        state.context.evm.env.block.prevrandao = Some(FixedBytes(rng.gen::<[u8; 32]>()));
    }

    /// Insert a user account into the DB
    ///
    /// # Arguments
    ///
    /// - `address` - Address to create the account at
    /// - `start_balance` - Starting balance of Eth of the account
    ///
    pub fn insert_account(&mut self, address: Address, start_balance: U256) {
        self.evm_state().context.evm.db.insert_account_info(
            address,
            AccountInfo::new(start_balance, 0, B256::default(), Bytecode::default()),
        );
    }

    /// Insert multiple accounts into the DB
    ///
    /// # Arguments
    ///
    /// - `start_balance` - Starting balance of Eth of the account
    /// - `addresses` - Vector of addresses to create accounts at
    ///
    pub fn insert_accounts(&mut self, start_balance: u128, addresses: Vec<Address>) {
        let start_balance = U256::from(start_balance);
        for address in addresses {
            self.insert_account(address, start_balance);
        }
    }

    /// Deploy a contract to the EVM
    ///
    /// # Arguments
    ///
    /// - `deployer` - Address of contract deployer
    /// - `contract_name` - Name of the contract, only used for
    ///   error messaging
    /// - `data` - Deployment bytecode, and abi encoded arguments
    ///   if required
    ///
    pub fn deploy_contract(
        &mut self,
        deployer: Address,
        contract_name: &str,
        data: Vec<u8>,
    ) -> Address {
        let tx = utils::init_create_transaction(deployer, data);
        let mut evm = self.evm();
        let result = evm.execute(tx);
        let output = utils::deployment_output(contract_name, result);
        let deploy_address = match output {
            revm::primitives::Output::Create(_, address) => address.unwrap(),
            _ => panic!("Deployment of {} failed", contract_name),
        };
        debug!("Deployed {} to {}", contract_name, deploy_address);
        self.evm_state = Some(evm.into_context_with_handler_cfg());
        deploy_address
    }

    /// Execute a contract function with ABI encoded arguments
    ///
    /// # Arguments
    ///
    /// - `callee` - Contract caller address
    /// - `contract` -  Address of the contract to call
    /// - `encoded_args` - ABI encoded function arguments and
    ///   function selector
    /// - `value` - Value attached to the transaction
    ///
    pub fn direct_execute_raw(
        &mut self,
        callee: Address,
        contract: Address,
        encoded_args: Vec<u8>,
        value: U256,
    ) -> Result<ExecutionResult, RevertError> {
        let tx = utils::init_call_transaction(callee, contract, encoded_args, value);
        let mut evm = self.evm();
        let execution_result = evm.execute(tx);
        self.evm_state = Some(evm.into_context_with_handler_cfg());
        utils::result_to_raw_output(callee, execution_result)
    }

    /// Execute a contract function for a specific ABI
    ///
    /// # Arguments
    ///
    /// - `callee` - Function caller address
    /// - `contract` - Address of the contract
    /// - `call_args` - Function arguments wrapped in a
    ///   [SolCall] object
    /// - `value` - Value attached to the transaction
    pub fn direct_execute<T: SolCall>(
        &mut self,
        callee: Address,
        contract: Address,
        call_args: T,
        value: U256,
    ) -> Result<(<T as SolCall>::Return, Vec<Log>), utils::RevertError> {
        let function_name = T::SIGNATURE;
        let call_args = call_args.abi_encode();
        let tx = utils::init_call_transaction(callee, contract, call_args, value);
        let mut evm = self.evm();
        let execution_result = evm.execute(tx);
        let (output, events) = utils::result_to_output(function_name, callee, execution_result)?;
        let output_data = output.into_data();
        let decoded = T::abi_decode_returns(&output_data, true);
        let decoded = match decoded {
            Ok(x) => x,
            Err(e) => panic!("Decoding error from {} {:?}", function_name, e),
        };
        self.evm_state = Some(evm.into_context_with_handler_cfg());
        Ok((decoded, events))
    }

    /// Call a contract function without committing changes
    ///
    /// # Arguments
    ///
    /// - `callee` - Address of the function caller
    /// - `contract` - Address of the contract
    /// - `encoded_args` - ABI encoded function selector and arguments
    /// - `value` - Value attached to the transaction
    pub fn direct_call_raw(
        &mut self,
        callee: Address,
        contract: Address,
        encoded_args: Vec<u8>,
        value: U256,
    ) -> Result<ExecutionResult, RevertError> {
        let tx = utils::init_call_transaction(callee, contract, encoded_args, value);
        let mut evm = self.evm();
        let result = evm.call(tx);
        self.evm_state = Some(evm.into_context_with_handler_cfg());
        utils::result_to_raw_output(callee, result.result)
    }

    /// Call a contract function without committing changes
    ///
    /// # Arguments
    ///
    /// - `callee` - Address of the function caller
    /// - `contract` - Address of the contract
    /// - `call_args` - Function arguments wrapped in a
    ///   [SolCall] object
    /// - `value` - Value attached to the transaction
    pub fn direct_call<T: SolCall>(
        &mut self,
        callee: Address,
        contract: Address,
        call_args: T,
        value: U256,
    ) -> Result<(<T as SolCall>::Return, Vec<Log>), utils::RevertError> {
        let function_name = T::SIGNATURE;
        let call_args = call_args.abi_encode();
        let tx = utils::init_call_transaction(callee, contract, call_args, value);
        let mut evm = self.evm();
        let execution_result = evm.call(tx);
        let (output, events) =
            utils::result_to_output(function_name, callee, execution_result.result)?;
        let output_data = output.into_data();
        let decoded = T::abi_decode_returns(&output_data, true);
        let decoded = match decoded {
            Ok(x) => x,
            Err(_) => panic!("Decoding error from {}", function_name),
        };
        self.evm_state = Some(evm.into_context_with_handler_cfg());
        Ok((decoded, events))
    }

    /// Execute a function from a [Transaction] object
    ///
    /// This function is used during simulation execution
    /// to process [Transaction] submitted for execution by
    /// agents.
    ///
    /// # Arguments
    ///
    /// - `transaction` - Struct containing function call parameters
    /// - `step` - Simulation step number
    /// - `sequence` - Ordering of this transaction in the queue
    ///
    fn call_from_transaction(
        evm: &mut Evm<'_, (), D>,
        last_events: &mut Vec<Event>,
        transaction: Transaction,
        step: usize,
        sequence: usize,
    ) {
        debug!(
            "Calling {:?} of {}",
            transaction.function_selector, transaction.transact_to
        );
        let function_selector = transaction.function_selector;
        let check_call = transaction.checked;
        let tx = utils::init_call_transaction(
            transaction.callee,
            transaction.transact_to,
            transaction.args,
            transaction.value,
        );
        let execution_result = evm.execute(tx);
        let result = utils::result_to_output_with_events(
            step,
            sequence,
            function_selector,
            transaction.callee,
            execution_result,
            check_call,
        );
        last_events.push(result);
    }

    /// Process a queue of [Transaction]
    ///
    /// # Arguments
    /// - `transactions` - Vector of transactions
    /// - `step` - Step number of the simulation
    ///
    pub fn process_transactions<R: Rng>(
        &mut self,
        transactions: Vec<Transaction>,
        rng: &mut R,
        step: usize,
    ) {
        let transactions = self.validator.order_transactions(rng, transactions);

        let mut evm = self.evm();
        let mut events = Vec::<Event>::new();

        for (i, call) in transactions.into_iter().enumerate() {
            Self::call_from_transaction(&mut evm, &mut events, call, step, i);
        }
        self.evm_state = Some(evm.into_context_with_handler_cfg());
        self.last_events.extend(events);
    }

    /// Store events from the last block
    ///
    /// Move events generated in the last block
    /// into the historical storage.
    ///
    pub fn clear_events(&mut self) {
        self.event_history.append(&mut self.last_events);
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::utils;
    use alloy_primitives::{Address, Signed, Uint};
    use alloy_sol_types::{sol, SolValue};
    use rand::SeedableRng;
    use rand_xoshiro::Xoroshiro128StarStar;
    use rstest::*;

    sol!(
        TestContract,
        r#"[
            {
                "inputs": [
                    {
                        "internalType": "int256",
                        "name": "x",
                        "type": "int256"
                    }
                ],
                "stateMutability": "nonpayable",
                "type": "constructor"
            },
            {
                "inputs": [],
                "name": "getValue",
                "outputs": [
                    {
                        "internalType": "int256",
                        "name": "",
                        "type": "int256"
                    }
                ],
                "stateMutability": "view",
                "type": "function"
            },
            {
                "inputs": [
                    {
                        "internalType": "int256",
                        "name": "x",
                        "type": "int256"
                    }
                ],
                "name": "setValue",
                "outputs": [],
                "stateMutability": "nonpayable",
                "type": "function"
            }
        ]"#
    );

    #[fixture]
    fn deployment() -> (Env<LocalDB, RandomValidator>, Address, Address) {
        let mut network =
            Env::<LocalDB, RandomValidator>::init(U256::ZERO, U256::ZERO, RandomValidator {});

        let constructor_args = <i128>::abi_encode(&101);
        let bytecode_hex = "608060405234801561001057600080fd5b50\
        6040516102063803806102068339818101604052810190610032919061007a\
        565b80600081905550506100a7565b600080fd5b6000819050919050565b61\
        005781610044565b811461006257600080fd5b50565b600081519050610074\
        8161004e565b92915050565b6000602082840312156100905761008f61003f\
        565b5b600061009e84828501610065565b91505092915050565b6101508061\
        00b66000396000f3fe608060405234801561001057600080fd5b5060043610\
        6100365760003560e01c8063209652551461003b5780635093dc7d14610059\
        575b600080fd5b610043610075565b60405161005091906100a1565b604051\
        80910390f35b610073600480360381019061006e91906100ed565b61007e56\
        5b005b60008054905090565b8060008190555050565b600081905091905056\
        5b61009b81610088565b82525050565b60006020820190506100b660008301\
        84610092565b92915050565b600080fd5b6100ca81610088565b81146100d5\
        57600080fd5b50565b6000813590506100e7816100c1565b92915050565b60\
        0060208284031215610103576101026100bc565b5b60006101118482850161\
        00d8565b9150509291505056fea2646970667358221220d99fa7a11a5739cf\
        9f1c4e30ebbb603943f8e1e44a3b4c0c10c3ea53799a236d64736f6c634300\
        080a0033";

        let user_address = Address::from(Uint::from(999));
        network.insert_account(user_address, Eth::to_weth(100));

        let mut bytecode: Vec<u8> = utils::data_bytes_from_hex(bytecode_hex);
        bytecode.extend(constructor_args);
        let contract_address = network.deploy_contract(user_address, "test", bytecode);

        (network, contract_address, user_address)
    }

    #[rstest]
    fn direct_execute_and_call(deployment: (Env<LocalDB, RandomValidator>, Address, Address)) {
        let (mut network, contract_address, user_address) = deployment;

        let (v, _) = network
            .direct_call(
                user_address,
                contract_address,
                TestContract::getValueCall {},
                U256::ZERO,
            )
            .unwrap();

        assert_eq!(v._0.as_i64(), 101i64);

        let _ = network
            .direct_execute(
                user_address,
                contract_address,
                TestContract::setValueCall { x: Signed::ONE },
                U256::ZERO,
            )
            .unwrap();

        let (v, _) = network
            .direct_call(
                user_address,
                contract_address,
                TestContract::getValueCall {},
                U256::ZERO,
            )
            .unwrap();

        assert_eq!(v._0.as_i64(), 1i64);
    }

    #[rstest]
    fn processing_calls(deployment: (Env<LocalDB, RandomValidator>, Address, Address)) {
        let (mut network, contract_address, user_address) = deployment;

        let calls = vec![
            Transaction::basic(
                user_address,
                contract_address,
                TestContract::setValueCall {
                    x: Signed::try_from_be_slice(&303u128.to_be_bytes()).unwrap(),
                },
                true,
            ),
            Transaction::basic(
                user_address,
                contract_address,
                TestContract::setValueCall {
                    x: Signed::try_from_be_slice(&303u128.to_be_bytes()).unwrap(),
                },
                true,
            ),
        ];

        let mut rng = Xoroshiro128StarStar::seed_from_u64(101);

        network.process_transactions(calls, &mut rng, 1);

        let (v, _) = network
            .direct_call(
                user_address,
                contract_address,
                TestContract::getValueCall {},
                U256::ZERO,
            )
            .unwrap();

        assert_eq!(v._0.as_i64(), 303i64);
    }
}
