***************
Model Mechanics
***************

.. note::

   The mechanics of the ABM presented here are
   the intended usage of VERBS, but the intention
   is that it also offers tools that allow users
   the flexibility to implement different
   model types and approaches.

State
=====

The EVM is intended to be the ground truth of the
state of the simulation, and agents interact
via the EVM (i.e. agents states are not directly
shared with other agents directly during the
simulation).

Model Step
==========

The model is designed to update in discrete steps,
where each step of the model represents the
processing of a new block in the chain.

Each step of the simulation performs several steps:

* The :code:`update` function is called for each agent and
  any generated transaction submitted to environment.
* The next block is processed:

  * The time and block number are incremented
  * The queue of transactions is shuffled
  * The queue of transactions is process, updating
    the state of the EVM

* The :code:`record` function of each agent is called
  and any data appended to the simulation
  history.

Agents
======

Agents can implement the base class :py:class:`verbs.sim.BaseAgent`
but otherwise should implement :py:meth:`verbs.sim.BaseAgent.update`
and :py:meth:`verbs.sim.BaseAgent.record`.

The :py:meth:`verbs.sim.BaseAgent.update` method should take
Numpy random generator and simulation environment arguments
and return a list of transactions to submit to the next block.

.. warning::

   The intention is that simulation agents are treated
   as if they update in parallel inside a step, i.e.
   they independently submit transactions to a shared
   queue to be processed in the next block.

   As such agents should not directly update the state
   of the EVM inside the update function, though they
   can call contract functions without committing change
   e.g. to retrieve data/state from the EVM.

The :py:meth:`verbs.sim.BaseAgent.record` should return data
to be recorded over the course of the simulation.

.. note::

   An agent does not neccesarily have to represent a single
   user/entity as long as it returns a list of transactions
   when updated, e.g. it may be more computationally efficient
   for an "agent" to represent a homogenous set of agents.
