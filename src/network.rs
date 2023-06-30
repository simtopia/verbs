use crate::agent::AgentSet;
use crate::contract::{Call, ContractDefinition, DeployedContract, Transaction};
use crate::utils::eth_to_weth;
use ethers_core::abi::{Detokenize, Tokenize};
use ethers_core::types::Selector;
use revm::{
    db::{CacheDB, EmptyDB},
    primitives::{
        AccountInfo, Address, Bytecode, ExecutionResult, Output, ResultAndState, TransactTo, TxEnv,
        U256,
    },
    EVM,
};
use std::ops::Range;

fn address_from_hex(x: &str) -> Address {
    let address = x.strip_prefix("0x").unwrap();
    let address = hex::decode(address).expect("Decoding failed");
    Address::from_slice(address.as_slice())
}

pub struct Network {
    pub evm: EVM<CacheDB<EmptyDB>>,
    pub admin_address: Address,
    pub contracts: Vec<DeployedContract>,
}

impl Network {
    fn init(admin_address: &str) -> Self {
        let admin_address = address_from_hex(admin_address);
        let mut evm = EVM::new();
        let mut db = CacheDB::new(EmptyDB {});

        evm.env.cfg.limit_contract_code_size = Some(0x100000);
        evm.env.cfg.disable_eip3607 = true;
        evm.env.block.gas_limit = U256::MAX;

        let start_balance = eth_to_weth(10_000);

        db.insert_account_info(
            admin_address,
            AccountInfo::new(start_balance, 0, Bytecode::default()),
        );
        db.insert_account_info(
            Address::zero(),
            AccountInfo::new(start_balance, 0, Bytecode::default()),
        );

        evm.database(db);

        Self {
            evm,
            admin_address,
            contracts: Vec::new(),
        }
    }

    pub fn from_range(start_balance: u128, r: Range<u64>, admin_address: &str) -> Self {
        let mut network = Network::init(admin_address);
        let db = network.evm.db().unwrap();

        for n in r {
            let address = Address::from(n);
            db.insert_account_info(
                address,
                AccountInfo::new(U256::from(start_balance), 0, Bytecode::default()),
            );
        }

        network
    }

    pub fn from_agents(
        start_balance: u128,
        agents: &Vec<Box<dyn AgentSet>>,
        admin_address: &str,
    ) -> Self {
        let mut network = Network::init(admin_address);
        let db = network.evm.db().unwrap();

        for agent_set in agents {
            for address in agent_set.get_call_addresses() {
                db.insert_account_info(
                    address,
                    AccountInfo::new(U256::from(start_balance), 0, Bytecode::default()),
                );
            }
        }

        network
    }

    pub fn execute(&mut self, tx: TxEnv) -> ExecutionResult {
        self.evm.env.tx = tx;

        let execution_result = match self.evm.transact_commit() {
            Ok(val) => val,
            Err(_) => panic!("Execution failed"),
        };

        execution_result
    }

    pub fn call(&mut self, tx: TxEnv) -> ResultAndState {
        self.evm.env.tx = tx;

        let execution_result = match self.evm.transact() {
            Ok(val) => val,
            Err(_) => panic!("Call failed"),
        };

        execution_result
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
        let result = self.execute(tx);
        let output = result_to_output("manually_deploy", result);
        let deploy_address = match output {
            revm::primitives::Output::Create(_, address) => address.unwrap(),
            _ => panic!("Deployment of {} failed", contract.name),
        };

        let deployed_contract = DeployedContract {
            name: contract.name,
            abi: contract.abi,
            address: deploy_address,
        };

        self.insert_contract(deployed_contract);

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

                let account_changes = self.call(tx);
                let output = result_to_output("deploy", account_changes.result);
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

                let account = AccountInfo::new(eth_to_weth(1), 0, contract.bytecode);

                db.insert_account_info(contract.deploy_address, account);

                for (k, v) in x {
                    db.insert_account_storage(contract.deploy_address, k, v)
                        .unwrap_or_else(|_| {
                            panic!("Could not insert account {} storage during deployment", k)
                        });
                }
            }
        }

        let deployed_contract = DeployedContract {
            name: contract.name,
            abi: contract.abi,
            address: contract.deploy_address,
        };

        self.insert_contract(deployed_contract);

        return contract.deploy_address;
    }

    fn insert_contract(&mut self, contract: DeployedContract) {
        self.contracts.push(contract);
    }

    pub fn encode_transaction<T: Tokenize>(&mut self, transaction: Transaction<T>) -> Call {
        let contract = &self.contracts[transaction.contract_idx];
        let encoded_args = contract
            .abi
            .encode(transaction.function_name, transaction.args)
            .unwrap();

        Call {
            callee: transaction.callee,
            transact_to: contract.address,
            args: encoded_args,
        }
    }

    pub fn create_call<T: Tokenize>(
        &mut self,
        callee: Address,
        contract_idx: usize,
        function_name: &'static str,
        args: T,
    ) -> Call {
        let contract = &self.contracts[contract_idx];
        let args = contract.abi.encode(function_name, args).unwrap();

        Call {
            callee,
            transact_to: contract.address,
            args,
        }
    }

    fn unwrap_transaction<'a, T: Tokenize>(
        &mut self,
        callee: Address,
        contract_idx: usize,
        function_name: &'static str,
        args: T,
    ) -> TxEnv {
        let contract = &self.contracts[contract_idx];
        let encoded = contract.abi.encode(function_name, args).unwrap();

        TxEnv {
            caller: callee,
            gas_limit: u64::MAX,
            gas_price: U256::ZERO,
            gas_priority_fee: None,
            transact_to: TransactTo::Call(contract.address),
            value: U256::ZERO,
            data: encoded.0,
            chain_id: None,
            nonce: None,
            access_list: Vec::new(),
        }
    }

    fn unwrap_transaction_with_selector<'a, T: Tokenize>(
        &mut self,
        callee: Address,
        contract_idx: usize,
        selector: Selector,
        args: T,
    ) -> TxEnv {
        let contract = &self.contracts[contract_idx];
        let encoded = contract.abi.encode_with_selector(selector, args).unwrap();

        TxEnv {
            caller: callee,
            gas_limit: u64::MAX,
            gas_price: U256::ZERO,
            gas_priority_fee: None,
            transact_to: TransactTo::Call(contract.address),
            value: U256::ZERO,
            data: encoded.0,
            chain_id: None,
            nonce: None,
            access_list: Vec::new(),
        }
    }

    fn unwrap_call(call: Call) -> TxEnv {
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

    fn decode_output<D: Detokenize>(
        &mut self,
        contract_idx: usize,
        function_name: &'static str,
        output_data: bytes::Bytes,
    ) -> D {
        let contract = &self.contracts[contract_idx];
        contract
            .abi
            .decode_output(function_name, output_data)
            .unwrap()
    }

    fn decode_output_with_selector<D: Detokenize>(
        &mut self,
        contract_idx: usize,
        selector: Selector,
        output_data: bytes::Bytes,
    ) -> D {
        let contract = &self.contracts[contract_idx];
        contract
            .abi
            .decode_output_with_selector(selector, output_data)
            .unwrap()
    }

    pub fn direct_execute<D: Detokenize, T: Tokenize>(
        &mut self,
        callee: Address,
        contract_idx: usize,
        function_name: &'static str,
        args: T,
    ) -> D {
        let tx = self.unwrap_transaction(callee, contract_idx, function_name, args);

        let execution_result: ExecutionResult = self.execute(tx);
        let output = result_to_output(function_name, execution_result);
        let output_data = output.into_data();
        self.decode_output(contract_idx, function_name, output_data)
    }

    pub fn direct_execute_with_selector<T: Tokenize>(
        &mut self,
        callee: Address,
        contract_idx: usize,
        selector: Selector,
        args: T,
    ) {
        let tx = self.unwrap_transaction_with_selector(callee, contract_idx, selector, args);
        let execution_result: ExecutionResult = self.execute(tx);
        let output = result_to_output("Selected", execution_result);
        let output_data: bytes::Bytes = output.into_data();
        self.decode_output_with_selector(contract_idx, selector, output_data)
    }

    pub fn direct_call<D: Detokenize, T: Tokenize>(
        &mut self,
        callee: Address,
        contract_idx: usize,
        function_name: &'static str,
        args: T,
    ) -> D {
        let tx = self.unwrap_transaction(callee, contract_idx, function_name, args);

        let execution_result = self.call(tx);
        let execution_result = execution_result.result;
        let output = result_to_output(function_name, execution_result);
        let output_data: bytes::Bytes = output.into_data();
        self.decode_output(contract_idx, function_name, output_data)
    }

    pub fn call_contract<D: Detokenize, T: Tokenize>(&mut self, transaction: Transaction<T>) -> D {
        let tx = self.unwrap_transaction(
            transaction.callee,
            transaction.contract_idx,
            transaction.function_name,
            transaction.args,
        );

        let execution_result = self.execute(tx);
        let output = result_to_output(transaction.function_name, execution_result);
        let output_data = output.into_data();
        self.decode_output(
            transaction.contract_idx,
            transaction.function_name,
            output_data,
        )
    }

    fn call_from_call(&mut self, call: Call) {
        let tx = Network::unwrap_call(call);
        self.execute(tx);
    }

    pub fn process_transactions<D: Detokenize, T: Tokenize>(
        &mut self,
        transactions: Vec<Transaction<T>>,
    ) {
        for call in transactions {
            self.call_contract::<D, T>(call);
        }
    }

    pub fn process_calls(&mut self, calls: Vec<Call>) {
        for call in calls {
            self.call_from_call(call);
        }
    }

    pub fn get_contract_address(&self, contract_idx: usize) -> Address {
        self.contracts.get(contract_idx).unwrap().address
    }
}

fn result_to_output(function_name: &'static str, execution_result: ExecutionResult) -> Output {
    match execution_result {
        ExecutionResult::Success { output, .. } => output,
        ExecutionResult::Revert { output, .. } => {
            panic!(
                "Failed to call {} due to revert: {:?}",
                function_name, output
            )
        }
        ExecutionResult::Halt { reason, .. } => {
            panic!("Failed to call {} due to halt: {:?}", function_name, reason)
        }
    }
}
