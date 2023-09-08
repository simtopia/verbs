use crate::agent::AgentSetVec;
use crate::contract::{Call, CallResult, ContractDefinition, DeployedContract, Event};
use crate::utils::{address_from_hex, Cast, Eth};
use bytes::Bytes;
use ethers_contract::BaseContract;
use ethers_core::abi::{Detokenize, Tokenize};
use ethers_core::types::{Address as EthAddress, Selector};
use log::{debug, warn};
use revm::{
    db::{CacheDB, EmptyDB},
    primitives::{
        AccountInfo, Address, Bytecode, ExecutionResult, Log, Output, ResultAndState, TransactTo,
        TxEnv, U256,
    },
    EVM,
};
use std::{fmt, ops::Range};

pub struct Network {
    pub evm: EVM<CacheDB<EmptyDB>>,
    pub admin_address: Address,
    pub contracts: Vec<DeployedContract>,
    pub last_events: Vec<Event>,
    pub event_history: Vec<Event>,
}

trait CallEVM {
    fn execute(&mut self, tx: TxEnv) -> ExecutionResult;
    fn call(&mut self, tx: TxEnv) -> ResultAndState;
}

impl CallEVM for EVM<CacheDB<EmptyDB>> {
    fn execute(&mut self, tx: TxEnv) -> ExecutionResult {
        self.env.tx = tx;

        let execution_result = match self.transact_commit() {
            Ok(val) => val,
            Err(_) => panic!("Execution failed"),
        };

        execution_result
    }

    fn call(&mut self, tx: TxEnv) -> ResultAndState {
        self.env.tx = tx;

        let execution_result = match self.transact() {
            Ok(val) => val,
            Err(_) => panic!("Call failed"),
        };

        execution_result
    }
}

#[derive(Debug, Clone)]
pub struct RevertError {
    pub function_name: &'static str,
    output: Bytes,
}

impl fmt::Display for RevertError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Failed to call {} due to revert: {:?}",
            self.function_name, self.output
        )
    }
}

impl Network {
    pub fn insert_account(&mut self, address: Address, start_balance: U256) {
        self.evm.db().unwrap().insert_account_info(
            address,
            AccountInfo::new(start_balance, 0, Bytecode::default()),
        );
    }

    pub fn init(admin_address: &str) -> Self {
        let admin_address = address_from_hex(admin_address);
        let mut evm = EVM::new();
        let db = CacheDB::new(EmptyDB {});

        evm.env.cfg.limit_contract_code_size = Some(0x1000000);
        evm.env.cfg.disable_eip3607 = true;
        evm.env.block.gas_limit = U256::MAX;

        let start_balance = U256::to_weth(10_000);
        evm.database(db);

        let mut network = Self {
            evm,
            admin_address,
            contracts: Vec::new(),
            last_events: Vec::new(),
            event_history: Vec::new(),
        };

        network.insert_account(admin_address, start_balance);
        network.insert_account(Address::zero(), start_balance);

        network
    }

    pub fn from_range(start_balance: u128, r: Range<u64>, admin_address: &str) -> Self {
        let mut network = Network::init(admin_address);
        let start_balance = U256::from(start_balance);

        for n in r {
            network.insert_account(Address::from(n), start_balance);
        }

        network
    }

    pub fn from_agents(start_balance: u128, agents: &AgentSetVec, admin_address: &str) -> Self {
        let mut network = Network::init(admin_address);
        network.insert_agents(start_balance, agents);
        network
    }

    pub fn insert_agents(&mut self, start_balance: u128, agents: &AgentSetVec) {
        let start_balance = U256::from(start_balance);
        for agent_set in agents.0.as_slice() {
            for address in agent_set.get_call_addresses() {
                self.insert_account(address, start_balance);
            }
        }
    }

    pub fn manually_deploy_contract(&mut self, contract: ContractDefinition) -> Address {
        let tx = TxEnv {
            caller: self.admin_address,
            gas_limit: u64::MAX,
            gas_price: U256::ZERO,
            gas_priority_fee: None,
            transact_to: TransactTo::create(),
            value: U256::ZERO,
            data: contract.arguments,
            chain_id: None,
            nonce: None,
            access_list: Vec::new(),
        };
        let result = self.evm.execute(tx);
        let output = deployment_output(&contract.name, result);
        let deploy_address = match output {
            revm::primitives::Output::Create(_, address) => address.unwrap(),
            _ => panic!("Deployment of {} failed", contract.name),
        };

        self.insert_contract(contract.name, contract.abi, deploy_address);

        deploy_address
    }

    pub fn deploy_contract(&mut self, contract: ContractDefinition) -> Address {
        match contract.storage_values {
            None {} => {
                let tx = TxEnv {
                    caller: self.admin_address,
                    gas_limit: u64::MAX,
                    gas_price: U256::ZERO,
                    gas_priority_fee: None,
                    transact_to: TransactTo::create(),
                    value: U256::ZERO,
                    data: contract.arguments,
                    chain_id: None,
                    nonce: None,
                    access_list: Vec::new(),
                };

                let account_changes = self.evm.call(tx);
                let output = deployment_output(&contract.name, account_changes.result);
                let deploy_address = match output {
                    revm::primitives::Output::Create(_, address) => address.unwrap(),
                    _ => panic!("Deployment of {} failed", contract.name),
                };

                let db = self.evm.db().unwrap();

                // Check we are not deploying to an already existing address
                if db.accounts.contains_key(&contract.deploy_address) {
                    panic!("Account at {} already exists", contract.deploy_address);
                };

                for (k, v) in account_changes.state.into_iter() {
                    if k == deploy_address {
                        db.insert_account_info(contract.deploy_address, v.info);
                        let storage_changes: hashbrown::HashMap<U256, U256> = v
                            .storage
                            .into_iter()
                            .map(|(k, v)| (k.clone(), v.present_value().clone()))
                            .collect();
                        db.replace_account_storage(contract.deploy_address, storage_changes)
                            .unwrap_or_else(|_| {
                                panic!("Could not update account {} storage during deployment", k)
                            });
                    } else {
                        for (ks, vs) in v.storage {
                            db.insert_account_storage(k, ks, vs.present_value())
                                .unwrap_or_else(|_| {
                                    panic!(
                                        "Could not insert account {} storage during deployment",
                                        k
                                    )
                                });
                        }
                    }
                }
            }
            Some(x) => {
                let db = self.evm.db().unwrap();

                // Check we are not deploying to an already existing address
                if db.accounts.contains_key(&contract.deploy_address) {
                    panic!("Account at {} already exists", contract.deploy_address);
                };

                let account = AccountInfo::new(U256::to_weth(1), 0, contract.bytecode);

                db.insert_account_info(contract.deploy_address, account);

                for (k, v) in x {
                    db.insert_account_storage(contract.deploy_address, k, v)
                        .unwrap_or_else(|_| {
                            panic!("Could not insert account {} storage during deployment", k)
                        });
                }
            }
        }

        self.insert_contract(contract.name, contract.abi, contract.deploy_address);
        return contract.deploy_address;
    }

    pub fn insert_contract(&mut self, name: String, abi: BaseContract, address: Address) {
        debug!("Inserted {} contract at {}", name, address);

        let contract = DeployedContract {
            name,
            abi,
            address,
            arg_address: address.cast(),
        };

        self.contracts.push(contract);
    }

    pub fn create_call<T: Tokenize>(
        &self,
        callee: Address,
        contract_idx: usize,
        function_name: &'static str,
        args: T,
        checked: bool,
    ) -> Call {
        let contract = self.contracts.get(contract_idx).unwrap();
        let args = match contract.abi.encode(function_name, args) {
            Ok(bytes) => bytes,
            Err(err) => panic!("Error encoding arguments to {}: {:?}", function_name, err),
        };

        Call {
            function_name,
            contract_idx,
            callee,
            transact_to: contract.address,
            args,
            checked,
        }
    }

    pub fn direct_execute<D: Detokenize, T: Tokenize>(
        &mut self,
        callee: Address,
        contract_idx: usize,
        function_name: &'static str,
        args: T,
    ) -> Result<(D, Vec<Log>), RevertError> {
        let contract = self.contracts.get(contract_idx).unwrap();
        let tx = contract.unwrap_transaction(callee, function_name, args);
        let execution_result: ExecutionResult = self.evm.execute(tx);
        let (output, events) = result_to_output(function_name, execution_result)?;
        let output_data = output.into_data();
        let decoded_output = contract.decode_output::<D>(function_name, output_data);
        Ok((decoded_output, events))
    }

    pub fn direct_execute_with_selector<D: Detokenize, T: Tokenize>(
        &mut self,
        callee: Address,
        contract_idx: usize,
        selector: Selector,
        args: T,
    ) -> Result<(D, Vec<Log>), RevertError> {
        let contract = self.contracts.get(contract_idx).unwrap();
        let tx = contract.unwrap_transaction_with_selector(callee, selector, args);
        let execution_result: ExecutionResult = self.evm.execute(tx);
        let (output, events) = result_to_output("Selected", execution_result)?;
        let output_data: bytes::Bytes = output.into_data();
        let decoded_output = contract.decode_output_with_selector::<D>(selector, output_data);
        Ok((decoded_output, events))
    }

    pub fn direct_call<D: Detokenize, T: Tokenize>(
        &mut self,
        callee: Address,
        contract_idx: usize,
        function_name: &'static str,
        args: T,
    ) -> Result<(D, Vec<Log>), RevertError> {
        let contract = self.contracts.get(contract_idx).unwrap();
        let tx = contract.unwrap_transaction(callee, function_name, args);
        let execution_result = self.evm.call(tx);
        let execution_result = execution_result.result;
        let (output, events) = result_to_output(function_name, execution_result)?;
        let output_data: bytes::Bytes = output.into_data();
        let decoded_output = contract.decode_output::<D>(function_name, output_data);
        Ok((decoded_output, events))
    }

    fn call_from_call(&mut self, call: Call, step: i64) {
        let _contract = self.contracts.get(call.contract_idx).unwrap();
        let function_name = call.function_name;
        let contract_idx = call.contract_idx;
        let check_call = call.checked;
        let tx = DeployedContract::unwrap_call(call);
        let execution_result = self.evm.execute(tx);
        let result = result_to_output_with_events(
            step,
            contract_idx,
            function_name,
            execution_result,
            check_call,
        );
        match result.events {
            Some(event) => self.last_events.push(event),
            None => {}
        }
    }

    pub fn process_calls(&mut self, calls: Vec<Call>, step: i64) {
        for call in calls {
            self.call_from_call(call, step);
        }
    }

    pub fn get_contract_address(&self, contract_idx: usize) -> EthAddress {
        self.contracts.get(contract_idx).unwrap().arg_address
    }

    pub fn clear_events(&mut self) {
        self.event_history.append(&mut self.last_events);
    }

    pub fn decode_event<R: Detokenize>(&self, event_name: &'static str, event: &Event) -> (i64, R) {
        (
            event.step,
            self.contracts[event.contract_idx]
                .decode_event(event_name, event.logs.last().unwrap().to_owned()),
        )
    }

    /// Decode events of a specific type from the last events into actual data
    ///
    /// # Arguments
    ///
    /// * `function_name` - Name of the function that produced the events
    /// * `event_name` - Name of the actual event to decode
    ///
    pub fn process_last_events<R: Detokenize>(
        &self,
        function_name: &'static str,
        event_name: &'static str,
    ) -> Vec<(i64, R)> {
        self.last_events
            .iter()
            .filter(|x| x.function_name == function_name)
            .map(|x| self.decode_event(event_name, x))
            .collect()
    }
    /// Decode events of a specific type from the full event history into actual data
    ///
    /// # Arguments
    ///
    /// * `function_name` - Name of the function that produced the events
    /// * `event_name` - Name of the actual event to decode
    ///
    pub fn process_event_history<R: Detokenize>(
        &self,
        function_name: &'static str,
        event_name: &'static str,
    ) -> Vec<(i64, R)> {
        self.event_history
            .iter()
            .filter(|x| x.function_name == function_name)
            .map(|x| self.decode_event(event_name, x))
            .collect()
    }
}

fn result_to_output_with_events(
    step: i64,
    contract_idx: usize,
    function_name: &'static str,
    execution_result: ExecutionResult,
    checked: bool,
) -> CallResult {
    match execution_result {
        ExecutionResult::Success { output, logs, .. } => match output {
            Output::Call(_) => CallResult {
                success: true,
                output,
                events: Some(Event {
                    function_name,
                    contract_idx,
                    logs,
                    step,
                }),
            },
            Output::Create(..) => {
                panic!("Unexpected call to create contract during simulation.")
            }
        },
        ExecutionResult::Revert { output, .. } => {
            if checked {
                panic!(
                    "Failed to call {} due to revert: {:?}",
                    function_name, output
                )
            } else {
                warn!(
                    "Failed to call {} due to revert: {:?}",
                    function_name, output
                );
                CallResult {
                    success: false,
                    output: Output::Call(Bytes::default()),
                    events: None,
                }
            }
        }
        ExecutionResult::Halt { reason, .. } => {
            panic!("Failed to call {} due to halt: {:?}", function_name, reason)
        }
    }
}

fn result_to_output(
    function_name: &'static str,
    execution_result: ExecutionResult,
) -> Result<(Output, Vec<Log>), RevertError> {
    match execution_result {
        ExecutionResult::Success { output, logs, .. } => Ok((output, logs)),
        ExecutionResult::Revert { output, .. } => Err(RevertError {
            function_name,
            output,
        }),
        ExecutionResult::Halt { reason, .. } => {
            panic!("Failed to call {} due to halt: {:?}", function_name, reason)
        }
    }
}

fn deployment_output(contract_name: &String, execution_result: ExecutionResult) -> Output {
    match execution_result {
        ExecutionResult::Success { output, .. } => output,
        ExecutionResult::Revert { output, .. } => {
            panic!(
                "Failed to deploy {} due to revert: {:?}",
                contract_name, output
            )
        }
        ExecutionResult::Halt { reason, .. } => {
            panic!(
                "Failed to deploy {} due to halt: {:?}",
                contract_name, reason
            )
        }
    }
}
