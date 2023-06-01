use crate::network::SimulationEnvironment;
use rand::rngs::ThreadRng;

pub trait Agent {
    fn update(&mut self, rng: &mut ThreadRng, network: &mut SimulationEnvironment);
}
