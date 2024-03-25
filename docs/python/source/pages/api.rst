*************
API Reference
*************

.. autosummary::
   :toctree: generated/
   :template: custom-module-template.rst
   :recursive:

   verbs.abi
   verbs.batch_runner
   verbs.sim
   verbs.utils

The ``verbs.envs`` module also exposes simulation
environments implemented in Rust:

.. autosummary::
   :toctree: generated/
   :template: custom-class-template.rst
   :nosignatures:

   verbs.envs.EmptyEnvRandom
   verbs.envs.EmptyEnvGasPriority
   verbs.envs.ForkEnvRandom
   verbs.envs.ForkEnvGasPriority
