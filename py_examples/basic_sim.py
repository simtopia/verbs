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
        network,
    ):
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

    def record(self):
        return self.balance


def run(n_steps):

    net = verbs.envs.EmptyEnv(1234, "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266")

    erc20_abi = verbs.abi.get_abi("ERC20", erc20_contract.ERC20_ABI)

    erc20_address = erc20_abi.constructor.deploy(
        net, erc20_contract.ERC20_BYTECODE, [int(1e19)]
    )

    agents = [
        Agent(i + 100, erc20_address, erc20_abi, N_AGENTS) for i in range(N_AGENTS)
    ]

    erc20_abi.transfer.execute(
        net,
        net.admin_address,
        erc20_address,
        [agents[0].address, int(1e19)],
    )

    runner = verbs.sim.Sim(101, net, agents)

    results = runner.run(n_steps)
    return np.array(results)


if __name__ == "__main__":
    run(50)
