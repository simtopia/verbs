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
        env,
    ):
        if np.random.uniform() < self.activation_rate:

            self.balance = self.abi.balanceOf.call(
                env, self.address, self.token_contract, [self.address]
            )[0][0]

            if self.balance > 0:
                receiver = rng.choice(self.n_agents) + 100
                receiver = verbs.utils.int_to_address(receiver)
                amount = min(self.balance, 100_000)
                send_call = self.abi.transfer.transaction(
                    self.address, self.token_contract, [receiver, amount]
                )

                return [send_call]
            else:
                return []
        else:
            return []

    def record(self, _env):
        return self.balance


def init_func(*, bytecode, constructor_args):
    env = verbs.envs.EmptyEnv(1234)

    admin = verbs.utils.int_to_address(99999999)
    env.create_account(admin, int(1e19))

    erc20_abi = verbs.abi.get_abi("ERC20", erc20_contract.ERC20_ABI)
    erc20_address = erc20_abi.constructor.deploy(
        env, admin, erc20_contract.ERC20_BYTECODE, [int(1e19)]
    )

    return env.export_snapshot(), (erc20_address, admin)


def exec_func(snapshot, n_steps, seed, addresses, *, activation_rate):
    erc20_address, admin = addresses

    env = verbs.envs.EmptyEnv(seed, snapshot)
    erc20_abi = verbs.abi.get_abi("ERC20", erc20_contract.ERC20_ABI)

    agents = [
        Agent(i + 100, erc20_address, erc20_abi, N_AGENTS, activation_rate)
        for i in range(N_AGENTS)
    ]

    erc20_abi.transfer.execute(
        env,
        admin,
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
