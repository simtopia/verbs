use crate::calls;
use fastrand::Rng;
use revm::primitives::bitvec::macros::internal::funty::Fundamental;
use rust_sim::agent::AdminAgent;
use rust_sim::network::Network;

pub struct PriceAdminAgent {
    token_a_price: i128,
    token_b_price: i128,
    token_a_price_idx: usize,
    token_b_price_idx: usize,
    mu: f64,
    dt: f64,
    sigma: f64,
}

impl PriceAdminAgent {
    pub fn new(
        token_a_price: i128,
        token_b_price: i128,
        token_a_price_idx: usize,
        token_b_price_idx: usize,
        mu: f64,
        dt: f64,
        sigma: f64,
    ) -> Self {
        PriceAdminAgent {
            token_a_price,
            token_b_price,
            token_a_price_idx,
            token_b_price_idx,
            mu,
            dt,
            sigma,
        }
    }
}

impl AdminAgent for PriceAdminAgent {
    fn update(&mut self, rng: &mut Rng, network: &mut Network) {
        // Box-Mueller to get normal distributed values
        let x1 = rng.f64();
        let x2 = rng.f64();

        let y1 = -2f64 * f64::ln(x1);
        let y2 = 2f64 * std::f64::consts::PI * x2;

        let n1 = self.dt * f64::sqrt(y1) * f64::cos(y2);
        let n2 = self.dt * f64::sqrt(y1) * f64::sin(y2);

        let n = (self.mu - self.sigma.powf(2f64) / 2f64) * self.dt;

        let z1 = f64::exp(n + self.sigma * n1);
        let z2 = f64::exp(n + self.sigma * n2);

        let new_price_a = self.token_a_price.as_f64() * z1;
        let new_price_b = self.token_b_price.as_f64() * z2;

        self.token_a_price = new_price_a.as_i128();
        self.token_b_price = new_price_b.as_i128();

        calls::set_token_price(network, self.token_a_price_idx, self.token_a_price);
        calls::set_token_price(network, self.token_b_price_idx, self.token_b_price);
    }
}
