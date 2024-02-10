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


def sim_func(
    env,
    seed,
    n_steps,
    *,
    activation_rate,
    erc20_address,
    erc20_abi,
    admin_address,
):
    agents = [
        Agent(i + 100, erc20_address, erc20_abi, N_AGENTS, activation_rate)
        for i in range(N_AGENTS)
    ]

    erc20_abi.transfer.execute(
        env,
        admin_address,
        erc20_address,
        [agents[0].address, int(1e19)],
    )

    runner = verbs.sim.Sim(seed, env, agents)
    results = runner.run(n_steps)
    return results


def run(n_steps, n_samples):
    env = verbs.envs.EmptyEnv(1234)

    admin_address = verbs.utils.int_to_address(99999999)
    env.create_account(admin_address, int(1e19))

    erc20_abi = verbs.abi.get_abi("ERC20", erc20_contract.ERC20_ABI)
    erc20_address = erc20_abi.constructor.deploy(
        env, admin_address, erc20_contract.ERC20_BYTECODE, [int(1e19)]
    )

    snapshot = env.export_snapshot()

    batch_results = verbs.batch_runner.batch_run(
        sim_func,
        n_steps=n_steps,
        n_samples=n_samples,
        parameters_samples=[dict(activation_rate=0.1), dict(activation_rate=0.2)],
        snapshot=snapshot,
        erc20_address=erc20_address,
        erc20_abi=erc20_abi,
        admin_address=admin_address,
    )

    return batch_results


if __name__ == "__main__":
    run(50, 3)
