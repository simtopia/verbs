from verbs import abi, batch_runner, envs, sim, utils


def test_batch_runner(bytecode, constructor_args, test_abi, agent_type):

    test_abi = abi.get_abi("TEST", test_abi)

    env = envs.EmptyEnvRandom(1234)
    admin = utils.int_to_address(99)
    env.create_account(admin, int(1e19))
    contract_address = env.deploy_contract(
        admin, "test_contract", bytecode + constructor_args
    )
    snapshot = env.export_snapshot()

    def sim_func(env, seed, n_steps, *, address, abi, offset):
        agent = agent_type(1, address, abi, offset=offset)
        sim_runner = sim.Sim(seed, env, [agent])
        results = sim_runner.run(n_steps)
        return results

    batch_results = batch_runner.batch_run(
        sim_func,
        n_steps=10,
        n_samples=2,
        parameters_samples=[dict(offset=0), dict(offset=0), dict(offset=1)],
        snapshot=snapshot,
        n_jobs=1,
        address=contract_address,
        abi=test_abi,
    )

    assert isinstance(batch_results, list)
    assert len(batch_results) == 3
    assert batch_results[0]["params"] == batch_results[1]["params"]
    assert batch_results[0]["params"] == dict(offset=0)
    # We ran with the same parameters, so should get the same results
    assert batch_results[0]["samples"] == batch_results[1]["samples"]
    assert len(batch_results[0]["samples"]) == 2
    assert len(batch_results[0]["samples"][0]) == 10

    assert batch_results[0]["params"] != batch_results[2]["params"]
    assert batch_results[2]["params"] == dict(offset=1)
    # But different parameters should differ
    assert batch_results[1]["samples"] != batch_results[2]["samples"]
    assert len(batch_results[2]["samples"]) == 2
    assert len(batch_results[2]["samples"][0]) == 10
