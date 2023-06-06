use crate::conract::ContractCall;
use crate::network::SimulationEnvironment;
use ethers_core::abi::Tokenize;
use rand::rngs::ThreadRng;

pub trait Agent<T: Tokenize> {
    fn update(
        &mut self,
        rng: &mut ThreadRng,
        network: &mut SimulationEnvironment,
    ) -> Option<ContractCall<T>>;
}
