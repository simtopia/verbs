Batch Execution
===============

Typically we might want to execute batches of simulation across
random seeds and simulation parameter samples,
:py:meth:`verbs.sim.batch_runner.batch_run`
implements functionality to generate simulation samples in parallel.

A typical use-case is generating an initial environment snapshot from a remote fork
(which is generally quite slow) and then quickly generate a large batch of samples
from the snapshot.

The simulation environments for the samples can be initialised from
either a snapshot (using the :py:meth:`verbs.envs.ForkEnv.export_snapshot` method)
or a cache (generated using the :py:meth:`verbs.envs.ForkEnv.export_cache` method).
In this is example we will create an initial snapshot after manually deploying
a contract and minting some tokens:

.. code-block:: python

   env = verbs.EmptyEnv(1234)

   admin_address = verbs.utils.int_to_address(99999999)
   env.create_account(admin_address, int(1e19))

   erc20_abi = verbs.abi.get_abi("ERC20", ERC20_ABI)
   erc20_address = erc20_abi.constructor.deploy(
       env, admin_address, ERC20_BYTECODE, [int(1e19)]
   )

   snapshot = env.export_snapshot()

Batch execution requires a simulation execution function with the signature

.. code-block:: python

   def sim_func(
       env, seed, n_steps, **params, **sim_kwargs
   ) -> typing.Any:
       ...

In this example we will re-use the previous example

.. code-block:: python

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

which initialises a set of agents, transfers tokens to one of the agents, then
runs the simulation.

The sampling can then be run using the :py:meth:`verbs.sim.batch_runner.batch_run`
function, providing the ``sim_func`` function and a set of parameters to sample over

.. code-block:: python

   batch_results = batch_run(
       sim_func,
       n_steps=100,
       n_samples=10,
       parameters_samples=[
           dict(activation_rate=0.1), dict(activation_rate=0.2)
       ],
       snapshot=snapshot,
       erc20_address=erc20_address,
       erc20_abi=erc20_abi,
       admin_address=admin_address,
    )

The batch-runner will generate sample and random seed combinations, and
execute simulation across these combinations in parallel. In this example
it will generate 10 Monte-Carlo samples for each set of parameters (20
samples, 2 parameter sets x 10 random seeds) each run for 100 steps.

For convenience the results are returned grouped by the parameters used to
generate them, in this case they will have the structure

.. code-block:: python

   [
       {
           "params": {"activation_rate": 0.1},
           "samples": [
               # List of Monte-Carlo sample results
               ...
           ]
       },
       {
           "params": {"activation_rate": 0.2},
           "samples": [
               # List of Monte-Carlo sample results
               ...
           ]
       }
   ]
