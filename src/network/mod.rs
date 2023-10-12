mod utils;
use crate::agent::AgentSetVec;
use crate::contract::{Call, Event};
use crate::utils::{address_from_hex, Eth};
use alloy_primitives::{Address, Uint, B256, U256};
use alloy_sol_types::SolCall;
use log::debug;
use revm::db::{CacheDB, EmptyDB};
use revm::primitives::{AccountInfo, Bytecode, ExecutionResult, Log, ResultAndState, TxEnv};
use revm::EVM;
use std::ops::Range;
pub use utils::RevertError;

pub struct Network {
    pub evm: EVM<CacheDB<EmptyDB>>,
    pub admin_address: Address,
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

impl Network {
    pub fn insert_account(&mut self, address: Address, start_balance: U256) {
        self.evm.db().unwrap().insert_account_info(
            address,
            AccountInfo::new(start_balance, 0, B256::default(), Bytecode::default()),
        );
    }

    pub fn init(admin_address: &str) -> Self {
        let admin_address = address_from_hex(admin_address);
        let mut evm = EVM::new();
        let db = CacheDB::new(EmptyDB::default());

        evm.env.cfg.limit_contract_code_size = Some(0x1000000);
        evm.env.cfg.disable_eip3607 = true;
        evm.env.block.gas_limit = U256::MAX;

        let start_balance = U256::to_weth(10_000);
        evm.database(db);

        let mut network = Self {
            evm,
            admin_address,
            last_events: Vec::new(),
            event_history: Vec::new(),
        };

        network.insert_account(admin_address, start_balance);
        network.insert_account(Address::ZERO, start_balance);

        network
    }

    pub fn from_range(start_balance: u128, r: Range<u64>, admin_address: &str) -> Self {
        let mut network = Network::init(admin_address);
        let start_balance = U256::from(start_balance);

        for n in r {
            network.insert_account(Address::from(Uint::from(n)), start_balance);
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
            for address in agent_set.get_addresses() {
                self.insert_account(address, start_balance);
            }
        }
    }

    pub fn deploy_contract(&mut self, contract_name: &str, data: Vec<u8>) -> Address {
        let tx = utils::init_create_transaction(self.admin_address, data);
        let result = self.evm.execute(tx);
        let output = utils::deployment_output(contract_name, result);
        let deploy_address = match output {
            revm::primitives::Output::Create(_, address) => address.unwrap(),
            _ => panic!("Deployment of {} failed", contract_name),
        };
        debug!("Deployed {} to {}", contract_name, deploy_address);
        deploy_address
    }

    pub fn direct_execute<T: SolCall>(
        &mut self,
        callee: Address,
        contract: Address,
        function_name: &'static str,
        call_args: T,
    ) -> Result<(<T as SolCall>::Return, Vec<Log>), utils::RevertError> {
        let call_args = call_args.abi_encode();
        let tx = utils::init_create_call_transaction(callee, contract, call_args);
        let execution_result = self.evm.execute(tx);
        let (output, events) = utils::result_to_output(function_name, execution_result)?;
        let output_data = output.into_data();
        let decoded = T::abi_decode_returns(&output_data, true);
        let decoded = match decoded {
            Ok(x) => x,
            Err(_) => panic!("Decoding error from {}", function_name),
        };
        Ok((decoded, events))
    }

    pub fn direct_call<T: SolCall>(
        &mut self,
        callee: Address,
        contract: Address,
        function_name: &'static str,
        call_args: T,
    ) -> Result<(<T as SolCall>::Return, Vec<Log>), utils::RevertError> {
        let call_args = call_args.abi_encode();
        let tx = utils::init_create_call_transaction(callee, contract, call_args);
        let execution_result = self.evm.call(tx);
        let (output, events) = utils::result_to_output(function_name, execution_result.result)?;
        let output_data = output.into_data();
        let decoded = T::abi_decode_returns(&output_data, true);
        let decoded = match decoded {
            Ok(x) => x,
            Err(_) => panic!("Decoding error from {}", function_name),
        };
        Ok((decoded, events))
    }

    fn call_from_call(&mut self, call: Call, step: usize, sequence: usize) {
        let function_name = call.function_name;
        let check_call = call.checked;
        let tx = utils::init_create_call_transaction(call.callee, call.transact_to, call.args);
        let execution_result = self.evm.execute(tx);
        let result = utils::result_to_output_with_events(
            step,
            sequence,
            function_name,
            execution_result,
            check_call,
        );
        if let Some(event) = result.events {
            self.last_events.push(event)
        }
    }

    pub fn process_calls(&mut self, calls: Vec<Call>, step: usize) {
        for (i, call) in calls.into_iter().enumerate() {
            self.call_from_call(call, step, i);
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
    fn deployment() -> (Network, Address) {
        let mut network = Network::init(Address::from(Uint::from(999)).to_string().as_str());

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

        let mut bytecode: Vec<u8> = utils::data_bytes_from_hex(bytecode_hex);
        bytecode.extend(constructor_args);
        let contract_address = network.deploy_contract("test", bytecode);

        (network, contract_address)
    }

    #[rstest]
    fn direct_execute_and_call(deployment: (Network, Address)) {
        let (mut network, contract_address) = deployment;

        let (v, _) = network
            .direct_call(
                network.admin_address,
                contract_address,
                "test",
                TestContract::getValueCall {},
            )
            .unwrap();

        assert_eq!(v._0.as_i64(), 101i64);

        let _ = network
            .direct_execute(
                network.admin_address,
                contract_address,
                "test",
                TestContract::setValueCall { x: Signed::ONE },
            )
            .unwrap();

        let (v, _) = network
            .direct_call(
                network.admin_address,
                contract_address,
                "test",
                TestContract::getValueCall {},
            )
            .unwrap();

        assert_eq!(v._0.as_i64(), 1i64);
    }

    #[rstest]
    fn processing_calls(deployment: (Network, Address)) {
        let (mut network, contract_address) = deployment;

        let calls = vec![
            Call {
                function_name: "set_value",
                callee: network.admin_address,
                transact_to: contract_address,
                args: TestContract::setValueCall {
                    x: Signed::try_from_be_slice(&202u128.to_be_bytes()).unwrap(),
                }
                .abi_encode(),
                checked: true,
            },
            Call {
                function_name: "set_value",
                callee: network.admin_address,
                transact_to: contract_address,
                args: TestContract::setValueCall {
                    x: Signed::try_from_be_slice(&303u128.to_be_bytes()).unwrap(),
                }
                .abi_encode(),
                checked: true,
            },
        ];

        network.process_calls(calls, 1);

        let (v, _) = network
            .direct_call(
                network.admin_address,
                contract_address,
                "test",
                TestContract::getValueCall {},
            )
            .unwrap();

        assert_eq!(v._0.as_i64(), 303i64);
    }
}
