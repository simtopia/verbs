use crate::{DATA_PROVIDER_INDEX, ORACLE_INDEX, POOL_INDEX};
use ethabi::{Address, Uint};
use ethers_core::types::U256;
use rust_sim::contract::Call;
use rust_sim::network::Network;
use rust_sim::utils::inverse_convert_address;

pub fn supply_call(
    network: &mut Network,
    user_address: Address,
    token_address: Address,
    amount: u128,
) -> Call {
    network.create_call(
        inverse_convert_address(user_address),
        POOL_INDEX,
        "supply",
        (
            token_address,
            U256::from(amount),
            user_address,
            U256::zero(),
        ),
    )
}

pub fn borrow_call(
    network: &mut Network,
    user_address: Address,
    token_address: Address,
    amount: U256,
) -> Call {
    network.create_call(
        inverse_convert_address(user_address),
        POOL_INDEX,
        "borrow",
        (token_address, amount, 1u128, 0u16, user_address),
    )
}

pub fn liquidation_call(
    network: &mut Network,
    collateral_token_address: Address,
    debt_token_address: Address,
    user_address: Address,
    liquidator_address: Address,
    amount: U256,
) -> Call {
    network.create_call(
        inverse_convert_address(liquidator_address),
        POOL_INDEX,
        "LiquidationCall",
        (
            collateral_token_address,
            debt_token_address,
            user_address,
            amount,
            true,
        ),
    )
}

pub fn get_reserve_configuration_data(
    network: &mut Network,
    token_idx: usize,
) -> (Uint, Uint, Uint, Uint, Uint, bool, bool, bool, bool, bool) {
    network.direct_call(
        network.admin_address,
        DATA_PROVIDER_INDEX,
        "getReserveConfigurationData",
        network.get_contract_address(token_idx),
    )
}

pub fn get_asset_price(network: &mut Network, token_address: Address) -> Uint {
    network.direct_call(
        network.admin_address,
        ORACLE_INDEX,
        "getAssetPrice",
        token_address,
    )
}

pub fn get_user_data(
    network: &mut Network,
    account_address: Address,
) -> (Uint, Uint, Uint, Uint, Uint, Uint) {
    network.direct_call(
        network.admin_address,
        POOL_INDEX,
        "getUserAccountData",
        account_address,
    )
}

pub fn set_token_price(network: &mut Network, idx: usize, price: i128) {
    let _: bool = network.direct_call(network.admin_address, idx, "setValue", price);
}
