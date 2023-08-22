use crate::agent::AgentSet;
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
        AccountInfo, Address, Bytecode, ExecutionResult, Output, ResultAndState, TransactTo, TxEnv,
        U256,
    },
    EVM,
};
use std::ops::Range;

pub struct Network {
    pub evm: EVM<CacheDB<EmptyDB>>,
    pub admin_address: Address,
    pub contracts: Vec<DeployedContract>,
    pub events: Vec<Event>,
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
            events: Vec::new(),
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

    pub fn from_agents(
        start_balance: u128,
        agents: &Vec<Box<&mut dyn AgentSet>>,
        admin_address: &str,
    ) -> Self {
        let mut network = Network::init(admin_address);
        network.insert_agents(start_balance, agents);
        network
    }

    pub fn insert_agents(&mut self, start_balance: u128, agents: &Vec<Box<&mut dyn AgentSet>>) {
        let start_balance = U256::from(start_balance);
        for agent_set in agents {
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
        let args = contract.abi.encode(function_name, args).unwrap();

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
    ) -> D {
        let contract = self.contracts.get(contract_idx).unwrap();
        let tx = contract.unwrap_transaction(callee, function_name, args);
        let execution_result: ExecutionResult = self.evm.execute(tx);
        let output = result_to_output(function_name, execution_result, true);
        let output_data = output.into_data();
        contract.decode_output(function_name, output_data)
    }

    pub fn direct_execute_with_selector<T: Tokenize>(
        &mut self,
        callee: Address,
        contract_idx: usize,
        selector: Selector,
        args: T,
    ) {
        let contract = self.contracts.get(contract_idx).unwrap();
        let tx = contract.unwrap_transaction_with_selector(callee, selector, args);
        let execution_result: ExecutionResult = self.evm.execute(tx);
        let output = result_to_output("Selected", execution_result, true);
        let output_data: bytes::Bytes = output.into_data();
        contract.decode_output_with_selector(selector, output_data)
    }

    pub fn direct_call<D: Detokenize, T: Tokenize>(
        &mut self,
        callee: Address,
        contract_idx: usize,
        function_name: &'static str,
        args: T,
    ) -> D {
        let contract = self.contracts.get(contract_idx).unwrap();
        let tx = contract.unwrap_transaction(callee, function_name, args);
        let execution_result = self.evm.call(tx);
        let execution_result = execution_result.result;
        let output = result_to_output(function_name, execution_result, true);
        let output_data: bytes::Bytes = output.into_data();
        contract.decode_output(function_name, output_data)
    }

    fn call_from_call(&mut self, call: Call, step: i64) {
        let _contract = self.contracts.get(call.contract_idx).unwrap();
        let function_name = call.function_name;
        let contract_idx = call.contract_idx;
        let check_call = call.checked;
        let tx = DeployedContract::unwrap_call(call);
        let execution_result = self.evm.execute(tx);
        let mut result = result_to_output_with_events(
            step,
            contract_idx,
            function_name,
            execution_result,
            check_call,
        );
        self.events.append(&mut result.events)
    }

    pub fn process_calls(&mut self, calls: Vec<Call>, step: i64) {
        for call in calls {
            self.call_from_call(call, step);
        }
    }

    pub fn get_contract_address(&self, contract_idx: usize) -> EthAddress {
        self.contracts.get(contract_idx).unwrap().arg_address
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
                events: logs
                    .into_iter()
                    .map(|x| Event {
                        function_name,
                        contract_idx,
                        log: x,
                        step,
                    })
                    .collect(),
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
                    events: Vec::default(),
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
    checked: bool,
) -> Output {
    match execution_result {
        ExecutionResult::Success { output, .. } => output,
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
                Output::Call(Bytes::default())
            }
        }
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
