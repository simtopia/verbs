import typing

import numpy as np

from verbs import abi, sim, utils


def test_sim_run(env, bytecode, constructor_args, test_abi):

    test_abi = abi.get_abi("TEST", test_abi)

    class Agent:
        def __init__(self, i: int, contract: typing.List[int]):
            self.address = utils.int_to_address(i)
            self.contract = contract
            self.current = 0

        def update(
            self,
            rng: np.random.Generator,
            network,
        ):
            get_call = test_abi.getValue.encode([])
            current = network.call(
                self.address,
                self.contract,
                get_call,
                0,
            )
            self.current = test_abi.getValue.decode(current[0])[0]

            set_call = test_abi.setValue.encode([self.current + 1])

            return [sim.Call(self.address, self.contract, set_call, True)]

        def record(self):
            return self.current

    address = env.deploy_contract("test_contract", bytecode + constructor_args)

    agent = Agent(1, address)

    sim_runner = sim.Sim(101, env, [agent])

    sim_runner.run(10)
