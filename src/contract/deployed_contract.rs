use alloy_primitives::Address;
use alloy_sol_types::SolInterface;

pub struct DeployedContract<A: SolInterface> {
    /// Name of the contract.
    pub name: String,
    /// ABI contract object.
    pub functions: A,
    /// Address of the contract.
    pub address: Address,
}
