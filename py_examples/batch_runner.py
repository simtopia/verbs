import erc20_contract
import numpy as np

import verbs

N_AGENTS = 50


class Agent:
    def __init__(
        self, i: int, token_contract: bytes, abi, n_agents: int, activation_rate: float
    ):
        self.address = verbs.utils.int_to_address(i)
        self.token_contract = token_contract
        self.abi = abi
        self.n_agents = n_agents
        self.balance = 0
        self.activation_rate = activation_rate

    def update(
        self,
        rng: np.random.Generator,
        network,
    ):
        if np.random.uniform() < self.activation_rate:

            self.balance = self.abi.balanceOf.call(
                network, self.address, self.token_contract, [self.address]
            )[0][0]

            if self.balance > 0:
                receiver = rng.choice(self.n_agents) + 100
                receiver = verbs.utils.int_to_address(receiver)
                amount = min(self.balance, 100_000)
                send_call = self.abi.transfer.get_call(
                    self.address, self.token_contract, [receiver, amount]
                )

                return [send_call]
            else:
                return []
        else:
            return []

    def record(self):
        return self.balance


def init_func(*, bytecode, constructor_args):
    net = verbs.envs.EmptyEnv(1234, "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266")

    erc20_abi = verbs.abi.get_abi("ERC20", erc20_contract.ERC20_ABI)
    erc20_address = erc20_abi.constructor.deploy(
        net, erc20_contract.ERC20_BYTECODE, [int(1e19)]
    )

    return net.export_snapshot(), erc20_address


def exec_func(snapshot, n_steps, seed, erc20_address, *, activation_rate):
    env = verbs.envs.EmptyEnv(seed, "", snapshot)
    erc20_abi = verbs.abi.get_abi("ERC20", erc20_contract.ERC20_ABI)

    agents = [
        Agent(i + 100, erc20_address, erc20_abi, N_AGENTS, activation_rate)
        for i in range(N_AGENTS)
    ]

    erc20_abi.transfer.execute(
        env,
        env.admin_address,
        erc20_address,
        [agents[0].address, int(1e19)],
    )

    runner = verbs.sim.Sim(101, env, agents)
    results = runner.run(n_steps)
    return results


def run(n_steps, n_samples):
    b = verbs.batch_runner.BatchRunner(init_func, exec_func)

    batch_results = b.run(
        n_steps,
        n_samples,
        [dict(activation_rate=0.1), dict(activation_rate=0.2)],
        init_kwargs=dict(
            bytecode=erc20_contract.ERC20_BYTECODE, constructor_args=[int(1e19)]
        ),
        n_jobs=1,
    )

    return batch_results


if __name__ == "__main__":
    run(50, 3)
