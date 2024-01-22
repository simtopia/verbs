from verbs import abi, batch_runner, envs, sim, utils


def test_batch_runner(bytecode, constructor_args, test_abi, agent_type):

    test_abi = abi.get_abi("TEST", test_abi)

    def init_func(_bytecode, _constructor_args):
        env = envs.EmptyEnv(1234)
        admin = utils.int_to_address(99)
        env.create_account(admin, int(1e19))
        address = env.deploy_contract(
            admin, "test_contract", _bytecode + _constructor_args
        )
        return env.export_snapshot(), address

    def exec_func(snapshot, n_steps, seed, contract_address):
        env = envs.EmptyEnv(seed, snapshot)
        agent = agent_type(1, contract_address, test_abi)
        sim_runner = sim.Sim(seed, env, [agent])
        results = sim_runner.run(n_steps)
        return results

    b = batch_runner.BatchRunner(init_func, exec_func)

    batch_results = b.run(
        10,
        3,
        [dict(), dict()],
        init_kwargs=dict(_bytecode=bytecode, _constructor_args=constructor_args),
        n_jobs=1,
    )

    assert len(batch_results) == 2
    # We ran with the same parameters, so should get the same results
    assert batch_results[0]["samples"] == batch_results[1]["samples"]
