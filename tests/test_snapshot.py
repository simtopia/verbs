from verbs import abi, envs, sim, utils


def test_snapshot_is_consistent(env, bytecode, constructor_args, test_abi, agent_type):

    test_abi = abi.get_abi("TEST", test_abi)

    admin = utils.int_to_address(99)
    env.create_account(admin, int(1e19))

    address = env.deploy_contract(admin, "test_contract", bytecode + constructor_args)

    agent = agent_type(1, address, test_abi)

    sim_runner = sim.Sim(101, env, [agent])

    sim_runner.run(10)

    snapshot = env.export_snapshot()

    env_from_snapshot = envs.EmptyEnv(101, snapshot)
    new_snapshot = env_from_snapshot.export_snapshot()

    assert snapshot[0] == new_snapshot[0]

    assert sorted(snapshot[1], key=lambda x: x[0]) == sorted(
        new_snapshot[1], key=lambda x: x[0]
    )
    assert sorted(snapshot[2], key=lambda x: x[0]) == sorted(
        new_snapshot[2], key=lambda x: x[0]
    )
    assert snapshot[3] == new_snapshot[3]
    assert sorted(snapshot[4], key=lambda x: x[0]) == sorted(
        new_snapshot[4], key=lambda x: x[0]
    )
