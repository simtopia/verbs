from verbs import abi

INITIAL_VALUE = 101


def test_deploy_and_call_contract(env, bytecode, constructor_args, test_abi):

    # Deploy contract and get address
    address = env.deploy_contract("test_contract", bytecode + constructor_args)

    # Init contract abi
    a = abi.get_abi("ABI", test_abi)

    # Get current contract value
    result_0, _, _ = a.getValue.call(env, env.admin_address, address, [])
    assert result_0[0] == INITIAL_VALUE

    # Update contract value
    _, logs, _ = a.setValue.execute(env, env.admin_address, address, [202])
    assert len(logs) == 1

    log_value = a.ValueUpdated.decode(logs[0][1])
    assert log_value == (101, 202)

    # Get new contract value
    result_2, _, _ = a.getValue.call(env, env.admin_address, address, [])
    assert result_2[0] == 202


def test_sim_update(env, bytecode, constructor_args, test_abi):

    # Deploy contract and get address
    address = env.deploy_contract("test_contract", bytecode + constructor_args)

    # Init contract abi
    a = abi.get_abi("ABI", test_abi)

    # Get current contract value
    result_0, _, _ = a.getValue.call(env, env.admin_address, address, [])
    assert result_0[0] == INITIAL_VALUE

    # Submit contract update call
    call_args = a.setValue.encode([202])
    env.submit_transaction(env.admin_address, address, call_args, 0, True)

    env.process_block()

    assert env.step == 1

    # Get new value after block update
    result_2, _, _ = a.getValue.call(env, env.admin_address, address, [])
    assert result_2[0] == 202


def test_get_admin_address(env):
    moo = env.admin_address
    assert isinstance(moo, bytes)
    assert len(moo) == 20
