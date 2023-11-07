use alloy_primitives::{address, hex, Address};
use std::time::Duration;

/// The cheatcode handler address.
///
/// This is the same address as the one used in DappTools's HEVM.
/// It is calculated as:
/// `address(bytes20(uint160(uint256(keccak256('hevm cheat code')))))`
pub const CHEATCODE_ADDRESS: Address = address!("7109709ECfa91a80626fF3989D68f67F5b1DD12D");

/// The Hardhat console address.
///
/// See: <https://github.com/nomiclabs/hardhat/blob/master/packages/hardhat-core/console.sol>
pub const HARDHAT_CONSOLE_ADDRESS: Address = address!("000000000000000000636F6e736F6c652e6c6f67");

/// Stores the caller address to be used as *sender* account for:
/// - deploying Test contracts
/// - deploying Script contracts
///
/// Derived from `address(uint160(uint256(keccak256("foundry default caller"))))`,
/// which is equal to `0x1804c8AB1F12E6bbf3894d4083f33e07309d1f38`.
pub const CALLER: Address = address!("1804c8AB1F12E6bbf3894d4083f33e07309d1f38");

/// The default test contract address.
pub const TEST_CONTRACT_ADDRESS: Address = address!("b4c79daB8f259C7Aee6E5b2Aa729821864227e84");

/// Magic return value returned by the `assume` cheatcode.
pub const MAGIC_ASSUME: &[u8] = b"FOUNDRY::ASSUME";

/// Magic return value returned by the `skip` cheatcode.
pub const MAGIC_SKIP: &[u8] = b"FOUNDRY::SKIP";

/// The default CREATE2 deployer.
pub const DEFAULT_CREATE2_DEPLOYER: Address = address!("4e59b44847b379578588920ca78fbf26c0b4956c");
/// The initcode of the default CREATE2 deployer.
pub const DEFAULT_CREATE2_DEPLOYER_CODE: &[u8] = &hex!("604580600e600039806000f350fe7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe03601600081602082378035828234f58015156039578182fd5b8082525050506014600cf3");
/// The runtime code of the default CREATE2 deployer.
pub const DEFAULT_CREATE2_DEPLOYER_RUNTIME_CODE: &[u8] = &hex!("7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe03601600081602082378035828234f58015156039578182fd5b8082525050506014600cf3");
pub const NON_ARCHIVE_NODE_WARNING: &str = "\
It looks like you're trying to fork from an older block with a non-archive node which is not \
supported. Please try to change your RPC url to an archive node if the issue persists.";
/// Transaction identifier of System transaction types
pub const SYSTEM_TRANSACTION_TYPE: u64 = 126u64;
/// Arbitrum L1 sender address of the first transaction in every block.
/// `0x00000000000000000000000000000000000a4b05`
pub const ARBITRUM_SENDER: Address = address!("00000000000000000000000000000000000a4b05");
/// The system address, the sender of the first transaction in every block:
/// `0xdeaddeaddeaddeaddeaddeaddeaddeaddead0001`
///
/// See also <https://github.com/ethereum-optimism/optimism/blob/65ec61dde94ffa93342728d324fecf474d228e1f/specs/deposits.md#l1-attributes-deposited-transaction>
pub const OPTIMISM_SYSTEM_ADDRESS: Address = address!("deaddeaddeaddeaddeaddeaddeaddeaddead0001");
/// The dev chain-id, inherited from hardhat
pub const DEV_CHAIN_ID: u64 = 31337;
/// Alchemy free tier cups: <https://docs.alchemy.com/reference/pricing-plans>
pub const ALCHEMY_FREE_TIER_CUPS: u64 = 330;
/// Default request timeout for http requests
///
/// Note: this is only used so that connections, that are discarded on the server side won't stay
/// open forever. We assume some nodes may have some backoff baked into them and will delay some
/// responses. This timeout should be a reasonable amount of time to wait for a request.
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(45);
