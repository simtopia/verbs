import numpy as np

from verbs import abi, batch_runner, envs, sim, utils


def test_batch_runner(bytecode, constructor_args, test_abi):

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

            set_call = test_abi.setValue.get_call(
                self.address, self.contract, [self.current + 1]
            )

            return [set_call]

        def record(self):
            return self.current

    def init_func(_bytecode, _constructor_args):
        env = envs.EmptyEnv(1234, "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266")
        address = env.deploy_contract("test_contract", _bytecode + _constructor_args)
        return env.export_snapshot(), address

    def exec_func(snapshot, n_steps, seed, contract_address):
        env = envs.EmptyEnv(seed, "", snapshot)
        agent = Agent(1, contract_address)
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
