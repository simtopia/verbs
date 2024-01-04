**********************
Simulation Environment
**********************

The simulation environment is implemented in Rust with a
Python API to allow Python implemented agents to interact
with the simulated EVM. It also has functionality
to update the state of the simulation, and track logs
generated during execution.

Initialisation
==============

A simulation environment can be initialised with either
an empty EVM state, or forked from a remote endpoint.

.. code-block:: python

   env = verbs.envs.EmptyEnv(
       1234, "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
   )

will initialise an environment class with an empty EVM state
(i.e. with no accounts or contracts) and random
seed ``1234``. Likewise

.. code-block:: python

   env = verbs.envs.ForkEnv(
       url, 1234, 1000, "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
   )

will create an environment with a fork backend, with
random seed `1234` and from block number `1000`. This
environment requests data that does not exist locally
from the remote endpoint, and so can be used
to run a simulation from an existing deployment, e.g.
a calling a contract will fetch the contract code and
any required storage values from the remote.

.. warning::

   Since the fork environment makes requests from a remote
   endpoint it is generally significantly slower for
   simulation usage compared to a purely local EVM. To this
   end we provide functionality to create a snapshot of
   the EVM state and use this to execute a purely local
   simulation without the performance penalty.

Functionality
=============

Both classes provide a common interface to allow Python
to interact with and retrieve data from the Rust environment.

* Deploy contracts and user accounts
* Directly call end execute contracts
* Submit transactions to be processed in the next block
* Process the next simulated block
* Retrieve logs/events generated in the last block and
  over the course of the simulation

See :py:class:`verbs.envs.EmptyEnv` or :py:class:`verbs.envs.ForkEnv`
for full details of the API.
