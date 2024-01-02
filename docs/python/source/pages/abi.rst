***************
ABI & Contracts
***************

The Rust simulation environment (currently) expects
transactions to provide ABI encoded arguments
from Python. VERBS provides utilities to create
Python types representing contract ABI's, with
functionality to encode/decode data and also
directly interact with the EVM.

========
ABI Type
========

For example given parsed ABI JSON

.. code-block:: python

   abi_json = [
       {
           "name": "foo",
           "inputs": [
               {"internalType": "int256", "name": "a", "type": "int256"}
           ],
           "outputs": [],
           "stateMutability": "view",
           "type": "function",
       },
       {
           "name": "bar",
           "anonymous": False,
           "inputs": [
               {
                   "indexed": False,
                   "internalType": "int256",
                   "name": "b",
                   "type": "int256"
               }
            ],
            "type": "event"
       }
    ]

using

.. code-block:: python

   Abi = get_abi("Foo", abi_json)

will create a Python type with attributes :code:`foo`
and :code:`bar` which can then be used to encode and decode
arguments

.. code-block:: python

   encoded_args = Abi.foo.encode([1, 2, 3])
   ...
   result = Abi.foo.decode(encoded_results)
   ...
   event_data = Abi.bar.decode(encoded_event_data)

The ABI type also has convenience functions to create transactions
and to directly interact with the EVM (encoding and decoding data)

.. code-block:: python

   # Transaction submitted by agent inside a simulation step
   call = Abi.foo.get_call(sender, address, [1, 2, 3])
   # Directly call the EVM (without committing) and decode result
   result = Abi.foo.call(env, sender, address, [1, 2, 3])
   # Directly call the EVM (and commit results) and decode result
   result = Abi.foo.execute(env, sender, address, [1, 2, 3])

The ABI type also defines a constructor attributes that aids
deploying contracts

.. code-block:: python

   deploy_address = abi.constructor.deploy(
       env, bytecode_hex, constructor_args
   )

==============
Initialisation
==============

ABI types can be created from multiple formats

* Python JSON data structure (i.e. a list of dicts)
* A JSON string
* A JSON/ABI file

See :py:mod:`verbs.abi` for more details.
