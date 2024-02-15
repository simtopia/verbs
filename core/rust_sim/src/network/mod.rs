mod utils;
use crate::contract::{Event, Transaction};
use crate::utils::Eth;
use alloy_primitives::{Address, Uint, B256, U256};
use alloy_sol_types::SolCall;
use db::{ForkDb, LocalDB, Requests, DB};
pub use ethereum_types::U64;
use log::debug;
use revm::primitives::{AccountInfo, Bytecode, ExecutionResult, Log, ResultAndState, TxEnv};
use revm::EVM;
use std::ops::Range;
pub use utils::{create_call, decode_event, process_events, RevertError};

pub struct Network<D: DB> {
    pub evm: EVM<D>,
    pub last_events: Vec<Event>,
    pub event_history: Vec<Event>,
}

trait CallEVM {
    fn execute(&mut self, tx: TxEnv) -> ExecutionResult;
    fn call(&mut self, tx: TxEnv) -> ResultAndState;
}

impl<D: DB> CallEVM for EVM<D> {
    fn execute(&mut self, tx: TxEnv) -> ExecutionResult {
        self.env.tx = tx;

        match self.transact_commit() {
            Ok(val) => val,
            Err(_) => panic!("Execution failed"),
        }
    }

    fn call(&mut self, tx: TxEnv) -> ResultAndState {
        self.env.tx = tx;

        match self.transact() {
            Ok(val) => val,
            Err(_) => panic!("Call failed"),
        }
    }
}

impl Network<ForkDb> {
    pub fn init(node_url: &str, block_number: u64) -> Self {
        let db = ForkDb::new(node_url, block_number);
        let mut evm = EVM::new();

        evm.env.cfg.limit_contract_code_size = Some(0x1000000);
        evm.env.cfg.disable_eip3607 = true;
        evm.env.block.gas_limit = U256::MAX;
        evm.env.block.timestamp = U256::try_from(db.block.timestamp.as_u128()).unwrap();
        evm.env.block.number = match db.block.number {
            Some(n) => U256::try_from(n.as_u64()).unwrap(),
            None => U256::ZERO,
        };

        evm.database(db);

        Self {
            evm,
            last_events: Vec::new(),
            event_history: Vec::new(),
        }
    }

    pub fn get_request_history(&self) -> &Requests {
        match &self.evm.db {
            Some(db) => &db.requests,
            None => panic!("No DB set"),
        }
    }
}

impl Network<LocalDB> {
    pub fn init(timestamp: U256, block_number: U256) -> Self {
        let mut evm = EVM::new();
        let db = LocalDB::new();

        evm.env.cfg.limit_contract_code_size = Some(0x1000000);
        evm.env.cfg.disable_eip3607 = true;
        evm.env.block.gas_limit = U256::MAX;
        evm.env.block.timestamp = timestamp;
        evm.env.block.number = block_number;

        let start_balance = U256::to_weth(10_000);
        evm.database(db);

        let mut network = Self {
            evm,
            last_events: Vec::new(),
            event_history: Vec::new(),
        };

        network.insert_account(Address::ZERO, start_balance);

        network
    }

    pub fn from_range(
        timestamp: U256,
        block_number: U256,
        start_balance: u128,
        r: Range<u64>,
    ) -> Self {
        let mut network = Network::<LocalDB>::init(timestamp, block_number);
        let start_balance = U256::from(start_balance);

        for n in r {
            network.insert_account(Address::from(Uint::from(n)), start_balance);
        }

        network
    }

    pub fn from_agents(
        timestamp: U256,
        block_number: U256,
        start_balance: u128,
        agent_addresses: Vec<Address>,
    ) -> Self {
        let mut network = Network::<LocalDB>::init(timestamp, block_number);
        network.insert_agents(start_balance, agent_addresses);
        network
    }
}

impl<D: DB> Network<D> {
    pub fn insert_account(&mut self, address: Address, start_balance: U256) {
        self.evm.db().unwrap().insert_account_info(
            address,
            AccountInfo::new(start_balance, 0, B256::default(), Bytecode::default()),
        );
    }

    pub fn insert_agents(&mut self, start_balance: u128, addresses: Vec<Address>) {
        let start_balance = U256::from(start_balance);
        for address in addresses {
            self.insert_account(address, start_balance);
        }
    }

    pub fn deploy_contract(
        &mut self,
        deployer: Address,
        contract_name: &str,
        data: Vec<u8>,
    ) -> Address {
        let tx = utils::init_create_transaction(deployer, data);
        let result = self.evm.execute(tx);
        let output = utils::deployment_output(contract_name, result);
        let deploy_address = match output {
            revm::primitives::Output::Create(_, address) => address.unwrap(),
            _ => panic!("Deployment of {} failed", contract_name),
        };
        debug!("Deployed {} to {}", contract_name, deploy_address);
        deploy_address
    }

    pub fn direct_execute_raw(
        &mut self,
        callee: Address,
        contract: Address,
        encoded_args: Vec<u8>,
        value: U256,
    ) -> Result<ExecutionResult, RevertError> {
        let tx = utils::init_call_transaction(callee, contract, encoded_args, value);
        let execution_result = self.evm.execute(tx);
        utils::result_to_raw_output(callee, execution_result)
    }

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
        let execution_result = self.evm.execute(tx);
        let (output, events) = utils::result_to_output(function_name, callee, execution_result)?;
        let output_data = output.into_data();
        let decoded = T::abi_decode_returns(&output_data, true);
        let decoded = match decoded {
            Ok(x) => x,
            Err(_) => panic!("Decoding error from {}", function_name),
        };
        Ok((decoded, events))
    }

    pub fn direct_call_raw(
        &mut self,
        callee: Address,
        contract: Address,
        encoded_args: Vec<u8>,
        value: U256,
    ) -> Result<ExecutionResult, RevertError> {
        let tx = utils::init_call_transaction(callee, contract, encoded_args, value);
        let result = self.evm.call(tx);
        utils::result_to_raw_output(callee, result.result)
    }

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
        let execution_result = self.evm.call(tx);
        let (output, events) =
            utils::result_to_output(function_name, callee, execution_result.result)?;
        let output_data = output.into_data();
        let decoded = T::abi_decode_returns(&output_data, true);
        let decoded = match decoded {
            Ok(x) => x,
            Err(_) => panic!("Decoding error from {}", function_name),
        };
        Ok((decoded, events))
    }

    fn call_from_transaction(&mut self, transaction: Transaction, step: usize, sequence: usize) {
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
        let execution_result = self.evm.execute(tx);
        let result = utils::result_to_output_with_events(
            step,
            sequence,
            function_selector,
            transaction.callee,
            execution_result,
            check_call,
        );
        self.last_events.push(result)
    }

    pub fn process_transactions(&mut self, transactions: Vec<Transaction>, step: usize) {
        for (i, call) in transactions.into_iter().enumerate() {
            self.call_from_transaction(call, step, i);
        }
    }

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
    fn deployment() -> (Network<LocalDB>, Address, Address) {
        let mut network = Network::<LocalDB>::init(U256::ZERO, U256::ZERO);

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
    fn direct_execute_and_call(deployment: (Network<LocalDB>, Address, Address)) {
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
    fn processing_calls(deployment: (Network<LocalDB>, Address, Address)) {
        let (mut network, contract_address, user_address) = deployment;

        let calls = vec![
            Transaction {
                function_selector: TestContract::setValueCall::SELECTOR,
                callee: user_address,
                transact_to: contract_address,
                args: TestContract::setValueCall {
                    x: Signed::try_from_be_slice(&202u128.to_be_bytes()).unwrap(),
                }
                .abi_encode(),
                value: U256::ZERO,
                checked: true,
            },
            Transaction {
                function_selector: TestContract::setValueCall::SELECTOR,
                callee: user_address,
                transact_to: contract_address,
                args: TestContract::setValueCall {
                    x: Signed::try_from_be_slice(&303u128.to_be_bytes()).unwrap(),
                }
                .abi_encode(),
                value: U256::ZERO,
                checked: true,
            },
        ];

        network.process_transactions(calls, 1);

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
