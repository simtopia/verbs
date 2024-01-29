mod dai_abi;

use dai_abi::ABI as DAI_ABI;

use alloy_primitives::{Address, U256};
use fork_evm::ForkDb;
use rust_sim::network::Network;
use rust_sim::utils::address_from_hex;

pub fn main() {
    let args: Vec<String> = std::env::args().collect();

    let url_str = format!("https://eth-mainnet.g.alchemy.com/v2/{}", args[1]);

    let mut net = Network::<ForkDb>::init(&url_str, 0);

    let dai_address = address_from_hex("0x6B175474E89094C44Da98b954EedeAC495271d0F");

    let decimals = net
        .direct_call(
            Address::ZERO,
            dai_address,
            DAI_ABI::decimalsCall {},
            U256::ZERO,
        )
        .unwrap();

    let total_supply = net
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
