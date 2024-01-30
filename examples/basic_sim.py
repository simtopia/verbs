import erc20_contract
import numpy as np

import verbs

N_AGENTS = 50


class Agent:
    def __init__(
        self,
        i: int,
        token_contract: bytes,
        abi,
        n_agents: int,
    ):
        self.address = verbs.utils.int_to_address(i)
        self.token_contract = token_contract
        self.abi = abi
        self.n_agents = n_agents
        self.balance = 0

    def update(
        self,
        rng: np.random.Generator,
        env,
    ):
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

    def record(self, _env):
        return self.balance


def run(n_steps):

    env = verbs.envs.EmptyEnv(1234)

    admin = verbs.utils.int_to_address(99999999)
    env.create_account(admin, int(1e19))

    erc20_abi = verbs.abi.get_abi("ERC20", erc20_contract.ERC20_ABI)

    erc20_address = erc20_abi.constructor.deploy(
        env, admin, erc20_contract.ERC20_BYTECODE, [int(1e19)]
    )

    agents = [
        Agent(i + 100, erc20_address, erc20_abi, N_AGENTS) for i in range(N_AGENTS)
    ]

    erc20_abi.transfer.execute(
        env,
        admin,
        erc20_address,
        [agents[0].address, int(1e19)],
    )

    runner = verbs.sim.Sim(101, env, agents)

    results = runner.run(n_steps)
    return np.array(results)


if __name__ == "__main__":
    run(50)
