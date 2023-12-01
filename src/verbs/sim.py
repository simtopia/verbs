import typing

import numpy as np
from tqdm import trange

from verbs import EmptyEnv, ForkEnv


class Sim:
    def __init__(
        self,
        seed: int,
        network,
        agents: typing.Optional[typing.List[typing.Any]] = None,
    ):

        self.network = network
        if agents is None:
            self.agents = list()
        else:
            self.agents = agents

        self.rng = np.random.default_rng(seed)

    @classmethod
    def new(
        seed: int,
        admin_address: str,
        agents: typing.Optional[typing.List[typing.Any]] = None,
    ):
        net = EmptyEnv(seed, admin_address)

        return Sim(seed, net, agents)

    @classmethod
    def fork(
        node_url: str,
        seed: int,
        block_number: int,
        admin_address: str,
        agents: typing.Optional[typing.List[typing.Any]] = None,
    ):
        net = ForkEnv(node_url, seed, block_number, admin_address)
        return Sim(seed, net, agents)

    def run(self, n_steps: int) -> typing.List[typing.Any]:

        records = list()

        for _ in trange(n_steps):

            calls = list()

            for agent in self.agents:
                a_calls = agent.update(self.rng, self.network)
                calls.extend(a_calls)

            self.rng.shuffle(calls)

            for call in calls:
                self.network.submit_call(
                    call.sender, call.contract_address, call.encoded_args, call.checked
                )

            self.network.process_block()

            agent_records = [agent.record() for agent in self.agents]
            records.append(agent_records)

        return records
