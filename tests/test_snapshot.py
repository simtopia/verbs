import numpy as np

from verbs import abi, envs, sim, utils


def test_snapshot_is_consistent(env, bytecode, constructor_args, test_abi):

    test_abi = abi.get_abi("TEST", test_abi)

    class Agent:
        def __init__(self, i: int, contract: bytes):
            self.address = utils.int_to_address(i)
            self.contract = contract
            self.current = 0

        def update(
            self,
            rng: np.random.Generator,
            network,
        ):
            self.current = test_abi.getValue.call(
                network, self.address, self.contract, []
            )[0][0]

            set_call = test_abi.setValue.transaction(
                self.address, self.contract, [self.current + 1]
            )

            return [set_call]

        def record(self):
            return self.current

    address = env.deploy_contract("test_contract", bytecode + constructor_args)

    agent = Agent(1, address)

    sim_runner = sim.Sim(101, env, [agent])

    sim_runner.run(10)

    snapshot = env.export_snapshot()

    env_from_snapshot = envs.EmptyEnv(101, "", snapshot)
    new_snapshot = env_from_snapshot.export_snapshot()

    assert snapshot[0] == new_snapshot[0]
    assert snapshot[1] == new_snapshot[1]

    assert sorted(snapshot[2], key=lambda x: x[0]) == sorted(
        new_snapshot[2], key=lambda x: x[0]
    )
    assert sorted(snapshot[3], key=lambda x: x[0]) == sorted(
        new_snapshot[3], key=lambda x: x[0]
    )
    assert snapshot[4] == new_snapshot[4]
    assert sorted(snapshot[5], key=lambda x: x[0]) == sorted(
        new_snapshot[5], key=lambda x: x[0]
    )
