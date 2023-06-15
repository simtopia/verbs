use crate::conract::Call;
use crate::network::Network;
use fastrand::Rng;

pub trait Agent {
    fn update(&mut self, rng: &mut Rng, network: &mut Network) -> Option<Call>;
}

pub trait RecordedAgent<T> {
    fn record(&self) -> T;
}
