Basic Simulation
================

Here we will work through a simple Example
where agents randomly exchange an ERC20 token
between themselves

Imports
-------

We'll start by importing Verbs and Numpy (which we use
for random number generation)

.. code-block:: python

   import numpy as np
   import verbs

Agent Definition
-----------------

The agents in this simulation simply pick another agent in the
simulation at random, and send them tokens if they have them
available.

We initialise the agent with

* Its own address (converted from an integer)
* The address of the token
* The token abi class
* The number of the agents in the sim

.. note::

   Addresses are stored as a :py:class:`bytes` object.

.. code-block:: python

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

agents should implement :py:meth:`verbs.sim.BaseAgent.update` and
:py:meth:`verbs.sim.BaseAgent.record` methods.

The ``update`` method is called for each agent at the start of each step.
In this example the agent performs 2 steps

- Get their current token balance via a call to the EVM.
- If their balance is greater than zero then they transfer tokens to
  a randomly selected agent.

The ``update`` method returns a list of calls to process as part of the
next block, in this case returning a token transfer transaction.

.. caution::

   Agents should not directly update the EVM inside the ``update``
   function (though they can call the EVM to get data on state). The state
   of the EVM is updated when all calls submitted by agents are processed
   in the next block.

This looks like:

.. code-block:: python

   ...
       def update(
           self,
           rng: np.random.Generator,
           env,
       ):
           self.balance = self.abi.balanceOf.call(
               network, self.address, self.token_contract, [self.address]
           )[0][0]

           if self.balance > 0:
               receiver = rng.choice(self.n_agents) + 100
               receiver = verbs.utils.int_to_address(receiver)
               amount = min(self.balance, 100_000)
               send_transaction = self.abi.transfer.transaction(
                   self.address, self.token_contract, [receiver, amount]
               )

           return [send_transaction]

       else:
           return []

The ``record`` method of this agent simply returns the current
token balance of the agent. The results from the ``record`` method
are collected across the agents at each step.

.. code-block:: python

   ...
       def record(self, env):
           return self.balance

.. tip::

   An agent does not necessarily have to represent a single entity in a
   simulation, but could also represent a group of agents of the same
   type. In this case the agent can submit multiple calls from its
   update function from the set of agents it represents.

Initialise Simulation
---------------------

We first initialise the simulation environment, and deploy the token
contract (the token ABI and bytecode have been omitted for brevity)

.. code-block:: python

   env = verbs.EmptyEnvRandom(1234)

   erc20_abi = verbs.abi.get_abi("ERC20", ERC20_ABI)

   admin = verbs.utils.int_to_address(99999999)
   env.create_account(admin, int(1e19))

   erc20_address = erc20_abi.constructor.deploy(env, admin, ERC20_BYTECODE, [int(1e19)])

The constructor :code:`verbs.EmptyEnv` initialises an empty EVM with the seed
``1234``. We also create an admin account that will deploy the contract. The token is
initialise with an initial allotment of ``1e19`` wei (minted to the admin address).

Initialise Agents
-----------------

We initialise a set of agents with the token address and token ABI

.. code-block:: python

   agents = [
        Agent(i + 100, erc20_address, erc20_abi, N_AGENTS) for i in range(N_AGENTS)
    ]

    erc20_abi.transfer.execute(
        env,
        admin,
        erc20_address,
        [agents[0].address, int(1e19)],
    )

at this point we also directly execute a transaction which transfers the
newly minted tokens from the admin agent to the first agent in the set.

Run the Simulation
------------------

The environment and agents are wrapped in a :py:class:`verbs.sim.Sim`

.. code-block:: python

   runner = verbs.sim.Sim(101, env, agents)

and then we can run the simulation

.. code-block:: python

   results = runner.run(n_steps)
   results = np.array(results)

The sim runner returns a list of records for each agent at every step
of the simulation. In this case we can readily convert this into a Numpy
array representing a time-series of the balances of each agent over the
course of the simulation.
