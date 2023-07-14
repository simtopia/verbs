use ethabi::{short_signature, ParamType};
use ethers_core::{
    abi::{AbiEncode, Token},
    types::{Address, Bytes},
};
use rust_sim::contract::load_contract as _load_contract;
use rust_sim::network::Network;
use rust_sim::utils::{convert_address, data_bytes_from_hex, inverse_convert_address};

fn load_contract(
    network: &mut Network,
    contract_name: &str,
    deployment_args: Vec<Token>,
) -> Address {
    let contract_path = "examples/aave/contracts/";
    let abi_path = format!("{}.abi", contract_name);
    let params_path = format!("{}.json", contract_name);

    let contract = _load_contract(
        format!("{}{}", contract_path, abi_path).as_str(),
        format!("{}{}", contract_path, params_path).as_str(),
        Some(deployment_args),
    );

    let contract_address = network.manually_deploy_contract(contract);
    println!("Deployed {} to {}", contract_name, contract_address);
    convert_address(contract_address)
}

fn initialisation_data(address: Address) -> Bytes {
    let data_prefix = "c4d66de8000000000000000000000000".to_string();
    let address_str = address.encode_hex()[26..].to_string();
    let full_hex = format!("{}{}", data_prefix, address_str);
    let full_hex = full_hex.as_str();
    data_bytes_from_hex(full_hex)
}

pub fn deploy_contracts(mut network: Network) -> Network {
    let admin_address = convert_address(network.admin_address);

    // 0
    let _address_provider_registry_address = load_contract(
        &mut network,
        "PoolAddressesProviderRegistry",
        vec![Token::Address(admin_address)],
    );

    let _: () =
        network.direct_execute(network.admin_address, 0, "transferOwnership", admin_address);

    // 1
    let _supply_logic_address = load_contract(&mut network, "SupplyLogic", vec![]);

    // 2
    let _borrow_logic_address = load_contract(&mut network, "BorrowLogic", vec![]);

    // 3
    let _liquidation_logic_address = load_contract(&mut network, "LiquidationLogic", vec![]);

    // 4
    let _emode_logic_address = load_contract(&mut network, "EModeLogic", vec![]);

    // 5
    let _bridge_logic_addresse_logic = load_contract(&mut network, "BridgeLogic", vec![]);

    // 6
    let _configurator_logic_address = load_contract(&mut network, "ConfiguratorLogic", vec![]);

    // 7
    let _flash_loan_logic_address = load_contract(&mut network, "FlashLoanLogic", vec![]);

    // 8
    let _pool_logic_address = load_contract(&mut network, "PoolLogic", vec![]);

    // 9
    let treasury_controller_address = load_contract(
        &mut network,
        "Treasury-Controller",
        vec![Token::Address(admin_address)],
    );

    // 10
    let treasury_proxy_address = load_contract(&mut network, "TreasuryProxy", vec![]);

    // 11
    let treasury_address = load_contract(&mut network, "Treasury-Implementation", vec![]);

    // Initialize treasury
    let _: () = network.direct_execute(network.admin_address, 11, "initialize", Address::zero());
    let selector = short_signature(
        "initialize",
        &[ParamType::Address, ParamType::Address, ParamType::Bytes],
    );
    let _: () = network.direct_execute_with_selector(
        network.admin_address,
        10,
        selector,
        (
            treasury_address,
            admin_address,
            initialisation_data(treasury_controller_address),
        ),
    );

    // 12
    let faucet_address = load_contract(
        &mut network,
        "Faucet-Test",
        vec![Token::Address(admin_address), Token::Bool(false)],
    );

    // 13
    let token_a_address = load_contract(
        &mut network,
        "DAI-TestnetMintableERC20-Test",
        vec![
            Token::String("A".to_string()),
            Token::String("A".to_string()),
            Token::Uint(18.into()),
            Token::Address(faucet_address),
        ],
    );

    // 14
    let token_b_address = load_contract(
        &mut network,
        "DAI-TestnetMintableERC20-Test",
        vec![
            Token::String("B".to_string()),
            Token::String("B".to_string()),
            Token::Uint(18.into()),
            Token::Address(faucet_address),
        ],
    );

    // 15
    let aave_address = load_contract(
        &mut network,
        "AAVE-TestnetMintableERC20-Test",
        vec![
            Token::String("AVEE".to_string()),
            Token::String("AVEE".to_string()),
            Token::Uint(18.into()),
            Token::Address(faucet_address),
        ],
    );

    // 16
    let weth_address = load_contract(
        &mut network,
        "WETH-TestnetMintableERC20-Test",
        vec![
            Token::String("WETH".to_string()),
            Token::String("WETH".to_string()),
            Token::Address(faucet_address),
        ],
    );

    // 17
    let staked_aave_proxy_address = load_contract(&mut network, "StakeAave-Proxy", vec![]);

    // 18
    let staked_aave_v1_address = load_contract(
        &mut network,
        "StakeAave-REV-1-Implementation",
        vec![
            Token::Address(aave_address),
            Token::Address(aave_address),
            Token::Uint(3600.into()),
            Token::Uint(1800.into()),
            Token::Address(admin_address),
            Token::Address(admin_address),
            Token::Uint(3600000.into()),
        ],
    );

    // 19
    let staked_aave_v2_address = load_contract(
        &mut network,
        "StakeAave-REV-2-Implementation",
        vec![
            Token::Address(aave_address),
            Token::Address(aave_address),
            Token::Uint(3600.into()),
            Token::Uint(1800.into()),
            Token::Address(admin_address),
            Token::Address(admin_address),
            Token::Uint(3600000.into()),
            Token::Address(Address::zero()),
        ],
    );

    // 20
    let staked_aave_v3_address = load_contract(
        &mut network,
        "StakeAave-REV-3-Implementation",
        vec![
            Token::Address(aave_address),
            Token::Address(aave_address),
            Token::Uint(3600.into()),
            Token::Uint(1800.into()),
            Token::Address(admin_address),
            Token::Address(admin_address),
            Token::Uint(3600000.into()),
            Token::String("Staked AAVE".to_string()),
            Token::String("stkAAVE".to_string()),
            Token::Uint(18.into()),
            Token::Address(Address::zero()),
        ],
    );

    // Initialize stake AAVE
    let _: () = network.direct_execute(
        network.admin_address,
        17,
        "initialize",
        (
            staked_aave_v1_address,
            admin_address,
            data_bytes_from_hex("f6d2ee860000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000c00000000000000000000000000000000000000000000000000000000000000012000000000000000000000000000000000000000000000000000000000000000b5374616b65642041415645000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000773746b4141564500000000000000000000000000000000000000000000000000")
        )
    );
    let _: () = network.direct_execute(
        network.admin_address,
        17,
        "upgradeToAndCall",
        (staked_aave_v2_address, data_bytes_from_hex("8129fc1c")),
    );
    let _: () = network.direct_execute(
        network.admin_address,
        17,
        "upgradeToAndCall",
        (staked_aave_v3_address, data_bytes_from_hex("8129fc1c")),
    );

    // 21
    let address_provider_address = load_contract(
        &mut network,
        "PoolAddressesProvider-Test",
        vec![
            Token::String("TEST".to_string()),
            Token::Address(admin_address),
        ],
    );

    let _: () = network.direct_execute(
        network.admin_address,
        0,
        "registerAddressesProvider",
        (address_provider_address, 8080u128),
    );

    // 22
    let data_provider_address = load_contract(
        &mut network,
        "PoolDataProvider-Test",
        vec![Token::Address(address_provider_address)],
    );

    let _: () = network.direct_execute(
        network.admin_address,
        21,
        "setPoolDataProvider",
        data_provider_address,
    );

    // 23
    let token_a_aggregator_address = load_contract(
        &mut network,
        "MockAggregator",
        vec![Token::Int(30000000000u128.into())],
    );

    // 24
    let token_b_aggregator_address = load_contract(
        &mut network,
        "MockAggregator",
        vec![Token::Int(100000000u128.into())],
    );

    // 25
    let aave_aggregator_address = load_contract(
        &mut network,
        "AAVE-TestnetPriceAggregator-Test",
        vec![Token::Int(1970000000u128.into())],
    );

    // 26
    let weth_aggregator_address = load_contract(
        &mut network,
        "WETH-TestnetPriceAggregator-Test",
        vec![Token::Int(400000000000u128.into())],
    );

    // 27
    let pool_address = load_contract(
        &mut network,
        "Pool-Implementation",
        vec![Token::Address(address_provider_address)],
    );

    let _: () = network.direct_execute(
        network.admin_address,
        27,
        "initialize",
        address_provider_address,
    );

    let _: () = network.direct_execute(network.admin_address, 21, "setPoolImpl", pool_address);
    let pool_proxy_address: Address = network.direct_call(network.admin_address, 21, "getPool", ());

    // 28
    println!("Deployed Pool-Proxy to {}", pool_proxy_address);
    network.insert_contract(
        "Pool-Proxy".to_string(),
        network.contracts[27].abi.clone(),
        inverse_convert_address(pool_proxy_address),
    );

    // 29
    let pool_configurator_address =
        load_contract(&mut network, "PoolConfigurator-Implementation", vec![]);

    let _: () = network.direct_execute(
        network.admin_address,
        29,
        "initialize",
        address_provider_address,
    );

    let _: () = network.direct_execute(
        network.admin_address,
        21,
        "setPoolConfiguratorImpl",
        pool_configurator_address,
    );

    let pool_configurator_proxy_address: Address =
        network.direct_call(network.admin_address, 21, "getPoolConfigurator", ());

    // 30
    println!("Deployed Pool-Configurator-Proxy to {}", pool_proxy_address);
    network.insert_contract(
        "Pool-Configurator-Proxy".to_string(),
        network.contracts[29].abi.clone(),
        inverse_convert_address(pool_configurator_proxy_address),
    );

    // 31
    let reserves_setup_helper_address = load_contract(&mut network, "ReservesSetupHelper", vec![]);

    let _: () = network.direct_execute(network.admin_address, 21, "setACLAdmin", admin_address);

    // 32
    let acl_manager_address = load_contract(
        &mut network,
        "ACLManager-Test",
        vec![Token::Address(address_provider_address)],
    );

    let _: () = network.direct_execute(
        network.admin_address,
        21,
        "setACLManager",
        acl_manager_address,
    );

    let _: () = network.direct_execute(network.admin_address, 32, "addPoolAdmin", admin_address);
    let _: () = network.direct_execute(
        network.admin_address,
        32,
        "addEmergencyAdmin",
        admin_address,
    );

    // 33
    let oracle_address = load_contract(
        &mut network,
        "AaveOracle-Test",
        vec![
            Token::Address(address_provider_address),
            Token::Array(vec![
                Token::Address(aave_address),
                Token::Address(token_a_address),
                Token::Address(token_b_address),
                Token::Address(weth_address),
            ]),
            Token::Array(vec![
                Token::Address(aave_aggregator_address),
                Token::Address(token_a_aggregator_address),
                Token::Address(token_b_aggregator_address),
                Token::Address(weth_aggregator_address),
            ]),
            Token::Address(Address::zero()),
            Token::Address(Address::zero()),
            Token::Uint(100000000u128.into()),
        ],
    );

    let _: () = network.direct_execute(network.admin_address, 21, "setPriceOracle", oracle_address);

    let _: () = network.direct_execute(
        network.admin_address,
        30,
        "updateFlashloanPremiumTotal",
        9u128,
    );
    let _: () = network.direct_execute(
        network.admin_address,
        30,
        "updateFlashloanPremiumToProtocol",
        0u128,
    );

    // 34
    let emission_manager_address = load_contract(
        &mut network,
        "EmissionManager",
        vec![Token::Address(admin_address)],
    );

    // 35
    let rewards_controller_address = load_contract(
        &mut network,
        "IncentivesV2-Implementation",
        vec![Token::Address(emission_manager_address)],
    );

    let _: () = network.direct_execute(network.admin_address, 35, "initialize", Address::zero());

    let id =
        data_bytes_from_hex("703c2c8634bed68d98c029c18f310e7f7ec0e5d6342c590190b3cb8b3ba54532")
            .to_ascii_lowercase();
    let id = Token::FixedBytes(id);

    let _: () = network.direct_execute(
        network.admin_address,
        21,
        "setAddressAsProxy",
        (id.clone(), rewards_controller_address),
    );
    let rewards_controller_proxy_address: Address =
        network.direct_execute(network.admin_address, 21, "getAddress", id);

    // 36
    println!(
        "Deployed IncentivesV2-Proxy to {}",
        rewards_controller_proxy_address
    );
    network.insert_contract(
        "IncentivesV2-Proxy".to_string(),
        network.contracts[35].abi.clone(),
        inverse_convert_address(rewards_controller_proxy_address),
    );

    let _: () = network.direct_execute(
        network.admin_address,
        34,
        "setRewardsController",
        rewards_controller_proxy_address,
    );

    // 37
    let _pull_rewards_transfer_address = load_contract(
        &mut network,
        "PullRewardsTransferStrategy",
        vec![
            Token::Address(rewards_controller_proxy_address),
            Token::Address(admin_address),
            Token::Address(admin_address),
        ],
    );

    // 38
    let _staked_token_transfer_strategy_address = load_contract(
        &mut network,
        "StakedTokenTransferStrategy",
        vec![
            Token::Address(rewards_controller_proxy_address),
            Token::Address(admin_address),
            Token::Address(staked_aave_proxy_address),
        ],
    );

    let _: () = network.direct_execute(
        network.admin_address,
        34,
        "transferOwnership",
        admin_address,
    );

    // 39
    let a_token_address = load_contract(
        &mut network,
        "AToken-Test",
        vec![Token::Address(pool_proxy_address)],
    );

    let _: () = network.direct_execute(
        network.admin_address,
        39,
        "initialize",
        (
            pool_proxy_address,
            Address::zero(),
            Address::zero(),
            Address::zero(),
            0u128,
            "ATOKEN_IMPL".to_string(),
            "ATOKEN_IMPL".to_string(),
            data_bytes_from_hex("00"),
        ),
    );

    // 40
    let _delegation_aware_token_address = load_contract(
        &mut network,
        "DelegationAwareAToken-Test",
        vec![Token::Address(pool_proxy_address)],
    );

    let _: () = network.direct_execute(
        network.admin_address,
        40,
        "initialize",
        (
            pool_proxy_address,
            Address::zero(),
            Address::zero(),
            Address::zero(),
            0u128,
            "DELEGATION_AWARE_ATOKEN_IMPL".to_string(),
            "DELEGATION_AWARE_ATOKEN_IMPL".to_string(),
            data_bytes_from_hex("00"),
        ),
    );

    // 41
    let stable_debt_token_address = load_contract(
        &mut network,
        "StableDebtToken-Test",
        vec![Token::Address(pool_proxy_address)],
    );

    let _: () = network.direct_execute(
        network.admin_address,
        41,
        "initialize",
        (
            pool_proxy_address,
            Address::zero(),
            Address::zero(),
            0u128,
            "STABLE_DEBT_TOKEN_IMPL".to_string(),
            "STABLE_DEBT_TOKEN_IMPL".to_string(),
            data_bytes_from_hex("00"),
        ),
    );

    // 42
    let variable_debt_token_address = load_contract(
        &mut network,
        "VariableDebtToken-Test",
        vec![Token::Address(pool_proxy_address)],
    );

    let _: () = network.direct_execute(
        network.admin_address,
        42,
        "initialize",
        (
            pool_proxy_address,
            Address::zero(),
            Address::zero(),
            0u128,
            "VARIABLE_DEBT_TOKEN_IMPL".to_string(),
            "VARIABLE_DEBT_TOKEN_IMPL".to_string(),
            data_bytes_from_hex("00"),
        ),
    );

    // 43
    let reserve_strategy_volatile_one_address = load_contract(
        &mut network,
        "ReserveStrategy-rateStrategyVolatileOne",
        vec![
            Token::Address(pool_proxy_address),
            Token::Uint(450000000000000000000000000u128.into()),
            Token::Uint(0u128.into()),
            Token::Uint(70000000000000000000000000u128.into()),
            Token::Uint(3000000000000000000000000000u128.into()),
            Token::Uint(0u128.into()),
            Token::Uint(0u128.into()),
            Token::Uint(20000000000000000000000000u128.into()),
            Token::Uint(50000000000000000000000000u128.into()),
            Token::Uint(200000000000000000000000000u128.into()),
        ],
    );

    // 44
    let reserve_strategy_stable_one_address = load_contract(
        &mut network,
        "ReserveStrategy-rateStrategyStableOne",
        vec![
            Token::Address(pool_proxy_address),
            Token::Uint(900000000000000000000000000u128.into()),
            Token::Uint(0u128.into()),
            Token::Uint(40000000000000000000000000u128.into()),
            Token::Uint(600000000000000000000000000u128.into()),
            Token::Uint(20000000000000000000000000u128.into()),
            Token::Uint(600000000000000000000000000u128.into()),
            Token::Uint(20000000000000000000000000u128.into()),
            Token::Uint(50000000000000000000000000u128.into()),
            Token::Uint(200000000000000000000000000u128.into()),
        ],
    );

    // 45
    let reserve_strategy_stable_two_address = load_contract(
        &mut network,
        "ReserveStrategy-rateStrategyStableTwo",
        vec![
            Token::Address(pool_proxy_address),
            Token::Uint(800000000000000000000000000u128.into()),
            Token::Uint(0u128.into()),
            Token::Uint(40000000000000000000000000u128.into()),
            Token::Uint(750000000000000000000000000u128.into()),
            Token::Uint(20000000000000000000000000u128.into()),
            Token::Uint(750000000000000000000000000u128.into()),
            Token::Uint(20000000000000000000000000u128.into()),
            Token::Uint(50000000000000000000000000u128.into()),
            Token::Uint(200000000000000000000000000u128.into()),
        ],
    );

    let _: () = network.direct_execute(
        network.admin_address,
        30,
        "initReserves",
        vec![
            (
                a_token_address,
                stable_debt_token_address,
                variable_debt_token_address,
                18u8,
                reserve_strategy_volatile_one_address,
                aave_address,
                treasury_proxy_address,
                rewards_controller_proxy_address,
                "AAVE".to_string(),
                "aAAVE".to_string(),
                "AAVE Variable Debt".to_string(),
                "VariableDebtAAVE".to_string(),
                "AAVE Stable Debt".to_string(),
                "StableDebtAAVE".to_string(),
                data_bytes_from_hex("10"),
            ),
            (
                a_token_address,
                stable_debt_token_address,
                variable_debt_token_address,
                18u8,
                reserve_strategy_stable_one_address,
                token_a_address,
                treasury_proxy_address,
                rewards_controller_proxy_address,
                "A".to_string(),
                "aA".to_string(),
                "A Variable Debt".to_string(),
                "VariableDebtA".to_string(),
                "A Stable Debt".to_string(),
                "StableDebtA".to_string(),
                data_bytes_from_hex("10"),
            ),
            (
                a_token_address,
                stable_debt_token_address,
                variable_debt_token_address,
                18u8,
                reserve_strategy_stable_two_address,
                token_b_address,
                treasury_proxy_address,
                rewards_controller_proxy_address,
                "B".to_string(),
                "aB".to_string(),
                "B Variable Debt".to_string(),
                "VariableDebtB".to_string(),
                "B Stable Debt".to_string(),
                "StableDebtB".to_string(),
                data_bytes_from_hex("10"),
            ),
            (
                a_token_address,
                stable_debt_token_address,
                variable_debt_token_address,
                18u8,
                reserve_strategy_volatile_one_address,
                weth_address,
                treasury_proxy_address,
                rewards_controller_proxy_address,
                "WETH".to_string(),
                "aWETH".to_string(),
                "WETH Variable Debt".to_string(),
                "VariableDebtWETH".to_string(),
                "WETH Stable Debt".to_string(),
                "StableDebtWETH".to_string(),
                data_bytes_from_hex("10"),
            ),
        ],
    );

    let _: () = network.direct_execute(
        network.admin_address,
        32,
        "addRiskAdmin",
        reserves_setup_helper_address,
    );

    let _: () = network.direct_execute(
        network.admin_address,
        31,
        "configureReserves",
        (
            pool_configurator_proxy_address,
            vec![
                (
                    aave_address,
                    5000u128,
                    6500u128,
                    11000u128,
                    0u128,
                    0u128,
                    0u128,
                    false,
                    false,
                    true,
                ),
                (
                    token_a_address,
                    7500u128,
                    8000u128,
                    10500u128,
                    1000u128,
                    0u128,
                    0u128,
                    true,
                    true,
                    true,
                ),
                (
                    token_b_address,
                    8000u128,
                    8500u128,
                    10500u128,
                    1000u128,
                    0u128,
                    0u128,
                    true,
                    true,
                    true,
                ),
                (
                    weth_address,
                    8000u128,
                    8250u128,
                    10500u128,
                    1000u128,
                    0u128,
                    0u128,
                    true,
                    true,
                    true,
                ),
            ],
        ),
    );

    let _: () = network.direct_execute(
        network.admin_address,
        32,
        "removeRiskAdmin",
        reserves_setup_helper_address,
    );

    // 46
    let _weth_gateway_address = load_contract(
        &mut network,
        "WrappedTokenGatewayV3",
        vec![
            Token::Address(weth_address),
            Token::Address(admin_address),
            Token::Address(pool_proxy_address),
        ],
    );

    // 47
    let _wallet_balance_provider_address =
        load_contract(&mut network, "WalletBalanceProvider", vec![]);

    let _: () = network.direct_execute(
        network.admin_address,
        30,
        "setBorrowableInIsolation",
        (token_a_address, true),
    );
    let _: () = network.direct_execute(
        network.admin_address,
        30,
        "setBorrowableInIsolation",
        (token_b_address, true),
    );
    let _: () = network.direct_execute(
        network.admin_address,
        30,
        "setEModeCategory",
        (
            1u8,
            9800u16,
            9850u16,
            10100u16,
            Address::zero(),
            "Stable-EMode".to_string(),
        ),
    );

    let _: () = network.direct_execute(
        network.admin_address,
        30,
        "setAssetEModeCategory",
        (token_a_address, 1u128),
    );

    let _: () = network.direct_execute(
        network.admin_address,
        30,
        "setAssetEModeCategory",
        (token_b_address, 1u128),
    );

    let _: () = network.direct_execute(
        network.admin_address,
        30,
        "setLiquidationProtocolFee",
        (aave_address, 1000u128),
    );

    let _: () = network.direct_execute(
        network.admin_address,
        30,
        "setLiquidationProtocolFee",
        (token_a_address, 1000u128),
    );

    let _: () = network.direct_execute(
        network.admin_address,
        30,
        "setLiquidationProtocolFee",
        (token_b_address, 1000u128),
    );

    let _: () = network.direct_execute(
        network.admin_address,
        30,
        "setLiquidationProtocolFee",
        (weth_address, 1000u128),
    );

    let _: () = network.direct_execute(network.admin_address, 30, "setPoolPause", false);

    network
}
