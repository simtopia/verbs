from verbs import abi

INITIAL_VALUE = 101


def test_deploy_and_call_contract(env, bytecode, constructor_args, test_abi):

    # Deploy contract and get address
    address = env.deploy_contract("test_contract", bytecode + constructor_args)

    # Init contract abi
    a = abi.get_abi("ABI", test_abi)

    # Get current contract value
    call_0 = a.getValue.encode([])
    result_0 = env.call(env.admin_address, address, call_0, 0)
    result_0 = a.getValue.decode(result_0[0])

    assert result_0[0] == INITIAL_VALUE

    # Update contract value
    call_1 = a.setValue.encode([202])
    _, logs, _ = env.execute(env.admin_address, address, call_1, 0)

    assert len(logs) == 1

    log_value = a.ValueUpdated.decode(logs[0][1])

    assert log_value == (101, 202)

    # Get new contract value
    call_2 = a.getValue.encode([])
    result_2 = env.call(env.admin_address, address, call_2, 0)
    result_2 = a.getValue.decode(result_2[0])
    assert result_2[0] == 202


def test_sim_update(env, bytecode, constructor_args, test_abi):

    # Deploy contract and get address
    address = env.deploy_contract("test_contract", bytecode + constructor_args)

    # Init contract abi
    a = abi.get_abi("ABI", test_abi)

    # Get current contract value
    call_0 = a.getValue.encode([])
    result_0 = env.call(env.admin_address, address, call_0, 0)
    result_0 = a.getValue.decode(result_0[0])
    assert result_0[0] == INITIAL_VALUE

    # Submit contract update call
    call_1 = a.setValue.encode([202])
    env.submit_call(env.admin_address, address, call_1, True)

    env.process_block()

    assert env.step == 1

    # Get new value after block update
    call_2 = a.getValue.encode([])
    result_2 = env.call(env.admin_address, address, call_2, 0)
    result_2 = a.getValue.decode(result_2[0])
    assert result_2[0] == 202


def test_get_admin_address(env):
    moo = env.admin_address
    assert isinstance(moo, bytes)
    assert len(moo) == 20
