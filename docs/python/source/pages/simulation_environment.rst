**********************
Simulation Environment
**********************

The simulation environment is implemented in Rust with a
Python API to allow Python implemented agents to interact
with the simulated EVM. It also has functionality
to update the state of the simulation, and track logs
generated during execution.

Simulation environments are also parametrised by a
*validator* that decides how transactions are ordered
for processing in each simulated block/step. Currently
two methods are supported:

- Randomly shuffle transactions
- Order transactions by nonce an gas-priority fee, each
  step

  - Transactions are grouped by sender address
  - Each group is individually sorted by nonce
  - Groups are sorted by the priority fee of the first transaction
  - The groups are flattened into a single queue of transactions

Initialisation
==============

The EVM state of the simulation environment is stored as
local in memory data structures. This in-memory database
can be initialised in several ways dependent on the use
case:

Empty Database
--------------

In this case the DB contains no data
and so the simulation should manually deploy contracts and
protocols. This is generally ok for small tests and
protocols, but can be labour intensive for larger protocols.

The simulation environment is then just initialised with a
random seed

.. code-block:: python

   # Using random transaction sorting
   env = verbs.envs.EmptyEnvRandom(1234)
   # Using gas-priority transaction ordering
   env = verbs.envs.EmptyEnvGasPriority(1234)

Remote fork
-----------

In this case the DB can request
data not present in a local database from a remote endpoint
(e.g. alchemy). For example if the EVM requires the bytecode
and storage values of a contract to execute a transaction in
the simulation, and they are not available locally, it will
attempt to retrieve data from the remote endpoint. This
means you can run simulations against actual deployed protocols,
but this comes at a performance cost, given the need to
retrieve data (possibly over the internet). For complex
protocols this may mean a large number of requests are required
to load sub-contracts and their data.

The environment is initialised with a endpoint url, random seed
and block number, for example:

.. code-block:: python

   # Using random transaction sorting
   env = verbs.envs.ForkEnvRandom(url, 1234, block_number=1000)
   # Using gas-priority transaction ordering
   env = verbs.envs.ForkEnvGasPriority(url, 1234, block_number=1000)

will create an environment with a fork backend, with
random seed `1234`, from block number `1000`.

.. warning::

   This simulation environment is *significantly* slower than
   a purely in memory db!

Snapshot
--------

The environment DB state can be initialised from
a snapshot of the state from a previous simulation. This can
either be used to start a simulation from a warm-start or to
continue a previous simulation. Note that this snapshot contains
the full state of the EVM including the states of any agents in
the simulation.

A snapshot can be created from a environment, and then directly
used to initialise an :py:class:`verbs.envs.EmptyEnv`

.. code-block:: python

   env = verbs.envs.ForkEnvRandom(url, 1234, block_number=1000)
   # Initialise & run a simulation
   snapshot = env.export_snapshot()
   # Use this snapshot to initialise a new environment
   new_env = verbs.envs.EmptyEnvRandom(1234, snapshot=snapshot)

Cache
-----

The overcome the performance drawbacks of the simulation, the
values requested during a simulation can be stored, and then
used to initialise values in a purely local database. This
should mean that any values that were requested (e.g. contracts
and their storage values) should be present, but not data
manually deployed (e.g. agent accounts or manually deployed
contracts).

This can be used to generate required data from an initially
slow simulation, but then subsequent simulations using the
cache can be run without the performance penalty.

This can be initialised by first creating a
:py:class:`verbs.envs.ForkEnv` and then creating a cache that
can be used to directly initialise a
:py:class:`verbs.envs.EmptyEnv`, for example

.. code-block:: python

   env = verbs.envs.ForkEnvRandom(url, 1234, block_number=1000)
   # Initialise & run a simulation
   ...
   # Export the cached requests
   cache = env.export_cache()
   # Use this cache to initialise a new environment
   faster_env = verbs.envs.EmptyEnvRandom(1234, cache=cache)

.. warning::

   This assumes that the initial simulation will request
   all the data required for subsequent simulations, i.e.
   that subsequent simulations call the same contracts/functions
   as the initial simulation. Missing data will lead to
   the simulation crashing or unexpected behaviour.

Functionality
=============

Both classes provide a common interface to allow Python
to interact with and retrieve data from the Rust environment.

* Deploy contracts and user accounts
* Directly call end execute contract functions
* Submit transactions to be processed in the next block
* Process the next simulated block
* Retrieve logs/events generated in the last block and
  over the course of the simulation

See

- :py:class:`verbs.envs.EmptyEnvRandom`
- :py:class:`verbs.envs.EmptyEnvGasPriority`
- :py:class:`verbs.envs.ForkEnvRandom`
- :py:class:`verbs.envs.ForkEnvGasPriority`

for full details of the API.
