mod dai_abi;

use alloy_primitives::{Address, U256};
use clap::Parser;
use dai_abi::ABI as DAI_ABI;
use rust_sim::network::Env;
use rust_sim::utils::address_from_hex;
use rust_sim::ForkDb;

#[derive(Parser, Debug)]
#[command(about, long_about = None)]
struct Args {
    /// Number of agents
    #[arg(short, long)]
    key: String,
}

pub fn main() {
    let args = Args::parse();

    let url_str = format!("https://eth-mainnet.g.alchemy.com/v2/{}", args.key);

    let mut env = Env::<ForkDb>::init(&url_str, 0);

    let dai_address = address_from_hex("0x6B175474E89094C44Da98b954EedeAC495271d0F");

    let decimals = env
        .direct_call(
            Address::ZERO,
            dai_address,
            DAI_ABI::decimalsCall {},
            U256::ZERO,
        )
        .unwrap();

    let total_supply = env
        .direct_call(
            Address::ZERO,
            dai_address,
            DAI_ABI::totalSupplyCall {},
            U256::ZERO,
        )
        .unwrap();

    println!("DAI decimals: {}", decimals.0._0);
    println!("DAI total supply: {}", total_supply.0._0)
}
