"""
Simulation runner and agent interface

Class wrapping simulation components, with functionality
to run the simulation and return telemetry.
"""

import typing

import numpy as np
from tqdm import trange

from verbs.envs import EmptyEnv, ForkEnv
from verbs.types import Cache, Env, Transaction


class BaseAgent:
    """
    Simulation agent interface/base-class

    Simulation agents should implement 2 methods:

    * ``update`` is called each for each agent for each step of
      the model, should update the state of the agent and
      return a list of transactions to process in the next block.
    * ``record`` is called at the end of each step, should return
      data to record over the course of the simulation

    .. note::

       Creating an agent does not automatically create a corresponding
       account im the EVM, this should be created using the
       :py:meth:`verbs.sim.BaseAgent.deploy` method, or
       :py:meth:`verbs.envs.EmptyEnv.create_account`.
    """

    def deploy(self, env: Env, address: bytes, eth: int):
        """
        Assign an address and create an account

        Assign this agent an address, and create a corresponding
        account in the EVM for use by this agent.

        Parameters
        ----------
        env: verbs.types.Env
            Simulation environment.
        address: bytes
            Address of the agent/account.
        eth: int
            Initial Eth to assign to this account (in units of wei).
        """
        self.address = address
        env.create_account(address, eth)

    def update(self, rng: np.random.Generator, env: Env) -> typing.List[Transaction]:
        """
        Update the state of the agent each step

        This method should **not** directly update the state
        of the EVM, and changes should be performed by returning
        a list of :class:`verbs.types.Transaction`. This method can however
        call the EVM without committing changes to the EVM, for example
        to retrieve data from contracts.

        Parameters
        ----------
        rng: numpy.random.Generator
            Numpy random generator, should be used for any random
            sampling to ensure determinism of the simulation.
        env: verbs.types.Env
            Network/EVM that the simulation interacts with.

        Returns
        -------
        typing.List[Transaction]
            List of transactions to be processed in the next block
            of the simulation. This can be an empty list if the
            agent is not submitting any transactions.
        """
        raise NotImplementedError

    def record(self, env: Env) -> typing.Any:
        """
        Record the state of the agent

        This method is called at the end of each step for all agents.
        It should return any data to be recorded over the course
        of the simulation.

        Parameters
        ----------
        env: verbs.types.Env
            Network/EVM that the simulation interacts with.

        Returns
        -------
        typing.Any
            Current recorded state for this agent.
        """
        raise NotImplementedError


class Sim:
    """
    Simulation state and execution class

    This class wraps the environment, agents and seeded
    random number generation. A sim can be initialised
    from either an empty environment (i.e one with no deployed
    contracts/accounts) of from a backend that fetches
    data from a remote fork.
    """

    def __init__(
        self,
        seed: int,
        env: Env,
        agents: typing.Optional[typing.List[BaseAgent]] = None,
    ):
        """
        Parameters
        ----------
        seed: int
            Random seed to initialise the simulation
            and key for use during execution
        network:
            Initialised simulation environment/network
        agents: typing.List[BaseAgent], optional
            List of agents to include in the simulation. Default
            value is an empty list, allowing agents to be pushed
            after the simulation is initialised.
        """
        self.env = env

        if agents is None:
            self.agents = list()
        else:
            self.agents = agents

        self.rng = np.random.default_rng(seed)

    @classmethod
    def empty(
        seed: int,
        agents: typing.Optional[typing.List[BaseAgent]] = None,
        snapshot=None,
        cache: Cache = None,
    ):
        """
        Initialise a simulation with an empty environment

        Initialise a simulation, initialising a fresh environment
        (i.e. one that contains no accounts, contracts etc.).

        Parameters
        ----------
        seed: int
            Random seed to initialise the simulation
            and key for use during execution.
        agents: typing.List[BaseAgent], optional
            List of agents to include in the simulation. Default
            value is an empty list, allowing agents to be pushed
            after the simulation is initialised.
        snapshot
            Optional snapshot used to initialise the simulation
            environment.
        cache
            Optional cache used to initialise the simulation
            environment.

        Returns
        -------
        Sim
            Initialised empty simulation.
        """
        env = EmptyEnv(seed, snapshot=snapshot, cache=cache)
        return Sim(seed, env, agents)

    @classmethod
    def fork(
        node_url: str,
        block_number: int,
        seed: int,
        agents: typing.Optional[typing.List[BaseAgent]] = None,
    ):
        """
        Initialise a simulation from a fork

        Initialise a simulation, initialising an environment using a forked
        backend. This backend can be used to fetch database values
        from a remote fork of the network state.

        Note
        ----
        Since the EVM in this simulation fetches data from a remote
        endpoint during execution, this simulation runner
        can be considerably slower than the purely in memory version.

        Parameters
        ----------
        node_url: str
            Url used to fetch data from, for example an alchemy API endpoint.
        block_number: int
            Number of the block to fetch data from, a value of ``0`` will
            mean the latest block will be retrieved.
        seed: int
            Random seed to initialise the simulation
            and key for use during execution.
        agents: typing.List[BaseAgent], optional
            List of agents to include in the simulation. Default
            value is an empty list, allowing agents to be pushed
            after the simulation is initialised.

        Returns
        -------
        Sim
            Initialised simulation with fork backend.
        """
        env = ForkEnv(node_url, seed, block_number)
        return Sim(seed, env, agents)

    def run(self, n_steps: int) -> typing.List[typing.List[typing.Any]]:
        """
        Run the simulation and return telemetry data

        This updates the simulation in fixed steps, inside each step:

        * The update function is called for all the agents, collecting
          the calls they submit for processing.
        * The calls are shuffled and processed through the EVM.
        * Records are gathered for each agent and appended to the
          sequence of records over the course of the simulation.

        Parameters
        ----------
        n_steps: int
            Number of steps (i.e. blocks) to run the simulation for.

        Returns
        -------
        typing.List[typing.List[typing.Any]]
            List of records collected from agents at each step of
            the simulation.
        """

        records = list()

        for _ in trange(n_steps):

            for agent in self.agents:
                calls = agent.update(self.rng, self.env)
                self.env.submit_transactions(calls)

            self.env.process_block()

            agent_records = [agent.record(self.env) for agent in self.agents]
            records.append(agent_records)

        return records
