import pytest

from verbs import abi, envs, utils

INITIAL_VALUE = 101


def test_deploy_and_call_contract(env, bytecode, constructor_args, test_abi):
    # Add deployment account
    admin = utils.int_to_address(99)
    env.create_account(admin, int(1e19))

    # Deploy contract and get address
    address = env.deploy_contract(admin, "test_contract", bytecode + constructor_args)

    # Init contract abi
    a = abi.get_abi("ABI", test_abi)

    # Get current contract value
    result_0, _, _ = a.getValue.call(env, admin, address, [])
    assert result_0[0] == INITIAL_VALUE

    # Update contract value
    _, logs, _ = a.setValue.execute(env, admin, address, [202])
    assert len(logs) == 1

    log_value = a.ValueUpdated.decode(logs[0][1])
    assert log_value == (101, 202)

    # Get new contract value
    result_2, _, _ = a.getValue.call(env, admin, address, [])
    assert result_2[0] == 202


def test_sim_update(env, bytecode, constructor_args, test_abi):
    # Add deployment account
    admin = utils.int_to_address(99)
    env.create_account(admin, int(1e19))

    # Deploy contract and get address
    address = env.deploy_contract(admin, "test_contract", bytecode + constructor_args)

    # Init contract abi
    a = abi.get_abi("ABI", test_abi)

    # Get current contract value
    result_0, _, _ = a.getValue.call(env, admin, address, [])
    assert result_0[0] == INITIAL_VALUE

    # Submit contract update call
    call_args = a.setValue.encode([202])
    env.submit_transaction(admin, address, call_args, True)

    env.process_block()

    assert env.step == 1

    # Get new value after block update
    result_2, _, _ = a.getValue.call(env, admin, address, [])
    assert result_2[0] == 202


def test_revert_exception(env, bytecode, constructor_args, test_abi):

    env.create_account(utils.ZERO_ADDRESS, int(1e19))

    a = abi.get_abi("ABI", test_abi)

    address = env.deploy_contract(
        utils.ZERO_ADDRESS, "test_contract", bytecode + constructor_args
    )

    # Should be fine
    a.setValue.execute(env, utils.ZERO_ADDRESS, address, [500])

    # Should revert as value is > 1000
    with pytest.raises(envs.RevertError):
        a.setValue.execute(env, utils.ZERO_ADDRESS, address, [1001])

    # Should also raise from a call
    with pytest.raises(envs.RevertError):
        a.setValue.call(env, utils.ZERO_ADDRESS, address, [1001])
