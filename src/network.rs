use crate::conract::{ContractDefinition, DeployedContract};
use ethers_core::abi::{Detokenize, Tokenize};
use revm::{
    db::{CacheDB, EmptyDB},
    primitives::{AccountInfo, Address, Bytecode, ExecutionResult, TransactTo, TxEnv, U256},
    EVM,
};

pub struct SimulationEnvironment {
    pub evm: EVM<CacheDB<EmptyDB>>,
    admin_address: Address,
    contracts: Vec<DeployedContract>,
}

impl SimulationEnvironment {
    pub fn new(start_balance: u128, n_users: u64) -> Self {
        let mut evm = EVM::new();
        let mut db = CacheDB::new(EmptyDB {});

        evm.env.cfg.limit_contract_code_size = Some(0x100000); // This is a large contract size limit, beware!
        evm.env.block.gas_limit = U256::MAX;

        for n in 0..n_users {
            let a = Address::from(n);
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
            // URGENT: change this to a custom error
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

        let execution_result = self.execute(tx);

        let output = match execution_result {
            ExecutionResult::Success { output, .. } => output,
            ExecutionResult::Revert { output, .. } => panic!("Failed due to revert: {:?}", output),
            ExecutionResult::Halt { reason, .. } => panic!("Failed due to halt: {:?}", reason),
        };
        let address = match output {
            revm::primitives::Output::Create(_, address) => address.unwrap(),
            _ => panic!("failed"),
        };

        let deployed_contract = DeployedContract {
            abi: contract.abi,
            address: address,
        };

        self.contracts.push(deployed_contract);

        return address;
    }

    pub fn call_contract<D: Detokenize, T: Tokenize>(
        &mut self,
        callee_idx: u64,
        contract_idx: usize,
        function_name: &str,
        args: T,
    ) -> D {
        let abi = self.contracts[contract_idx].abi.clone();
        let address = self.contracts[contract_idx].address.clone();
        let encoded = abi.encode(function_name, args).unwrap();

        let tx = TxEnv {
            caller: Address::from(callee_idx),
            gas_limit: u64::MAX,
            gas_price: U256::ZERO,
            gas_priority_fee: None,
            transact_to: TransactTo::Call(address),
            value: U256::ZERO,
            data: encoded.0,
            chain_id: None,
            nonce: None,
            access_list: Vec::new(),
        };

        let execution_result = self.execute(tx);

        let output = match execution_result {
            ExecutionResult::Success { output, .. } => output,
            ExecutionResult::Revert { output, .. } => panic!("Failed due to revert: {:?}", output),
            ExecutionResult::Halt { reason, .. } => panic!("Failed due to halt: {:?}", reason),
        };

        let output_data = output.into_data();
        let output = abi.decode_output(function_name, output_data).unwrap();

        return output;
    }
}
