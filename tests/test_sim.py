from verbs import abi, sim, utils


def test_sim_run(env, bytecode, constructor_args, test_abi, agent_type):

    test_abi = abi.get_abi("TEST", test_abi)

    # Add deployment account
    admin = utils.int_to_address(99)
    env.create_account(admin, int(1e19))

    address = env.deploy_contract(admin, "test_contract", bytecode + constructor_args)

    agent = agent_type(1, address, test_abi)

    sim_runner = sim.Sim(101, env, [agent])

    sim_runner.run(10)
