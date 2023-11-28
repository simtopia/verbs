use alloy_primitives::{address, Address};
use std::time::Duration;

pub const NON_ARCHIVE_NODE_WARNING: &str = "\
It looks like you're trying to fork from an older block with a non-archive node which is not \
supported. Please try to change your RPC url to an archive node if the issue persists.";
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
