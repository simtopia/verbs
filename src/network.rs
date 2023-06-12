use crate::conract::{ContractDefinition, DeployedContract, Transaction};
use ethers_core::abi::{Detokenize, Tokenize};
use revm::{
    db::{CacheDB, EmptyDB},
    primitives::{
        AccountInfo, Address, Bytecode, ExecutionResult, ResultAndState, TransactTo, TxEnv, U256,
    },
    EVM,
};

pub struct Network {
    pub evm: EVM<CacheDB<EmptyDB>>,
    admin_address: Address,
    contracts: Vec<DeployedContract>,
}

impl Network {
    pub fn new(start_balance: u128, n_users: usize) -> Self {
        let mut evm = EVM::new();
        let mut db = CacheDB::new(EmptyDB {});

        evm.env.cfg.limit_contract_code_size = Some(0x100000);
        evm.env.block.gas_limit = U256::MAX;

        for n in 0..n_users {
            let a = Address::from(u64::try_from(n).expect("Couldn't cast n_users to a u64"));
            db.insert_account_info(
                a,
                AccountInfo::new(U256::from(start_balance), 0, Bytecode::default()),
            );
        }

        evm.database(db);

        Self {
            evm: evm,
            admin_address: Address::from(0),
            contracts: Vec::new(),
        }
    }

    pub fn execute(&mut self, tx: TxEnv) -> ExecutionResult {
        self.evm.env.tx = tx;

        let execution_result = match self.evm.transact_commit() {
            Ok(val) => val,
            Err(_) => panic!("failed"),
        };

        execution_result
    }

    pub fn call(&mut self, tx: TxEnv) -> ResultAndState {
        self.evm.env.tx = tx;

        let execution_result = match self.evm.transact() {
            Ok(val) => val,
            Err(_) => panic!("failed"),
        };

        execution_result
    }

    pub fn deploy_contract(&mut self, contract: ContractDefinition) -> Address {
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

        let output = match account_changes.result {
            ExecutionResult::Success { output, .. } => output,
            ExecutionResult::Revert { output, .. } => panic!("Failed due to revert: {:?}", output),
            ExecutionResult::Halt { reason, .. } => panic!("Failed due to halt: {:?}", reason),
        };

        let deploy_address = match output {
            revm::primitives::Output::Create(_, address) => address.unwrap(),
            _ => panic!("failed"),
        };

        let db = self.evm.db().unwrap();

        for (k, v) in account_changes.state.into_iter() {
            if k == deploy_address {
                db.insert_account_info(contract.deploy_address, v.info);
                let storage_changes: hashbrown::HashMap<U256, U256> = v
                    .storage
                    .into_iter()
                    .map(|(k, v)| (k.clone(), v.present_value().clone()))
                    .collect();
                db.replace_account_storage(contract.deploy_address, storage_changes)
                    .expect("Doh");
            } else {
                for (ks, vs) in v.storage {
                    db.insert_account_storage(k, ks, vs.present_value())
                        .expect("Doo");
                }
            }
        }

        let deployed_contract = DeployedContract {
            abi: contract.abi,
            address: contract.deploy_address,
        };

        self.contracts.push(deployed_contract);

        return contract.deploy_address;
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

    pub fn call_without_commit<D: Detokenize, T: Tokenize>(
        &mut self,
        callee: Address,
        contract_idx: usize,
        function_name: &'static str,
        args: T,
    ) -> D {
        let tx = self.unwrap_transaction(callee, contract_idx, function_name, args);

        let execution_result = self.call(tx);
        let execution_result = execution_result.result;

        let output = match execution_result {
            ExecutionResult::Success { output, .. } => output,
            ExecutionResult::Revert { output, .. } => panic!("Failed due to revert: {:?}", output),
            ExecutionResult::Halt { reason, .. } => panic!("Failed due to halt: {:?}", reason),
        };

        let output_data = output.into_data();

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

        let output = match execution_result {
            ExecutionResult::Success { output, .. } => output,
            ExecutionResult::Revert { output, .. } => panic!("Failed due to revert: {:?}", output),
            ExecutionResult::Halt { reason, .. } => panic!("Failed due to halt: {:?}", reason),
        };

        let output_data = output.into_data();

        self.decode_output(
            transaction.contract_idx,
            transaction.function_name,
            output_data,
        )
    }

    pub fn process_transactions<D: Detokenize, T: Tokenize>(
        &mut self,
        transactions: Vec<Transaction<T>>,
    ) {
        for call in transactions {
            self.call_contract::<D, T>(call);
        }
    }
}
