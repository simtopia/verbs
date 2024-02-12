"""
Generate ABI types from ABI JSON

Convert ABI JSON into python types with functionality
for encoding/decoding function arguments and events
and generating calls to submit to the simulation
EVM.

The generated type has attributes representing the
functions and events of the abi, along with a constructor
attribute for use when deploying a contract.

Examples
--------

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

    Abi = get_abi("Foo", abi_json)

    # Encode arguments & selector to bytes
    encoded_args = Abi.foo.encode([10])
    # Decode bytes returned from a contract function call
    decoded_result = Abi.foo.decode(result_bytes)
    # Decode data attached to an event/log
    decoded_event = Abi.bar.decode(event_data)

Notes
-----

Overloaded functions (i.e. those with the same name) are mapped
to attributes with numbered suffixes according to their order
in the ABI, e.g. if the ABI contained two functions named ``foo``
the resulting type will have ``foo0`` and ``foo1`` attributes.
"""

import json
import typing

import eth_abi
import eth_utils

from verbs import types, utils


def parse_input_types(abi: typing.Dict) -> typing.List[str]:
    """
    Parses function / event input types necessary for encoding

    Parameters
    ----------
    abi: typing.Dict
        Parsed ABI JSON of the function or the event.

    Returns
    -------
    inputs: typing.List[str]
        list with the input types.
    """

    def tuple_parser(x: typing.List[str]):
        """
        Recursive tuple parser
        """
        inp = []
        for y in x:
            if y["type"] == "tuple":
                inp.append(tuple_parser(y["components"]))
            else:
                inp.append(y["type"])
        input_tuple = f"({','.join(inp)})"
        return input_tuple

    inputs = []
    for x in abi["inputs"]:
        indexed = x.get("indexed", False)
        if not indexed:
            if x["type"] == "tuple":
                inputs.append(tuple_parser(x["components"]))
            else:
                inputs.append(x["type"])
    return inputs


class Constructor:
    """
    ABI constructor class

    Class representing contracts constructor with
    functionality to encode arguments and to deploy
    the contract.

    Parameters
    ----------
    name: str
        ABI name (used for debugging/logging).
    abi: typing.Dict
        Parsed ABI JSON of the constructor.
    """

    def __init__(self, name: str, abi: typing.Dict):
        self.name = name
        self.inputs = [x["type"] for x in abi["inputs"]]

    def encode(self, args: typing.List[typing.Any]) -> bytes:
        """
        ABI encode constructor arguments

        Parameters
        ----------
        args: List[Any]
            List of argument values to the constructor.

        Returns
        -------
        bytes
            ABI encoded constructor arguments
        """
        return eth_abi.encode(self.inputs, args)

    def deploy(
        self, env, deployer: bytes, bytecode: str, args: typing.List = None
    ) -> bytes:
        """
        Deploy a contract

        Parameters
        ----------
        env: verbs.types.Env
            Simulation environment.
        bytecode: str
            Contract bytecode hex string.
        args: List[Any], optional
            Optional list of constructor argument values

        Returns
        -------
        bytes
            Address the contract was deployed to.
        """
        if args is None:
            args = []

        encoded_args = self.encode(args)
        deploy_address = env.deploy_contract(
            deployer, self.name, utils.hex_to_bytes(bytecode) + encoded_args
        )

        return deploy_address


class Function:
    """
    ABI function class

    Class representing a contract function with functionality
    to encode and decode calls and results to/from this function.

    Parameters
    ----------
    abi: typing.Dict
        Parsed ABI JSON of the function.

    Attributes
    ----------
    inputs: typing.List[str]
        List of function input types.
    outputs: typing.List[str]
        List of function output types.
    selector: bytes
        ABI encoded function selector

    Examples
    --------

    .. code-block:: python

       f_abi = {
           "name": "foo",
           "inputs": [
               {"internalType": "int256", "name": "a", "type": "int256"}
           ],
           "outputs": [],
           "stateMutability": "view",
           "type": "function",
       }

       f = Function(f_abi)
       encoded_args = f.encode([10])
       result = f.call(
           env, sender_address, contract_address, args
       )
    """

    def __init__(self, abi: typing.Dict):
        self.inputs = parse_input_types(abi)
        self.outputs = [x["type"] for x in abi["outputs"]]
        self.selector = eth_utils.abi.function_abi_to_4byte_selector(abi)

    def encode(self, args: typing.List[typing.Any]) -> bytes:
        """
        ABI encode function arguments

        Parameters
        ----------
        args: typing.List[typing.Any]
            List of function arguments.

        Returns
        -------
        bytes
            ABI encoded args with function selector.
        """
        return self.selector + eth_abi.encode(self.inputs, args)

    def decode(self, output: bytes) -> typing.Tuple[typing.Any, ...]:
        """
        Decode result(s) from this function

        Parameters
        ----------
        output: bytes
            Bytes returned from the EVM from a call to this function.

        Returns
        -------
        typing.Tuple[typing.Any]
            Tuple of decoded return values from this function.
        """
        return eth_abi.decode(self.outputs, bytes(output))

    def transaction(
        self,
        sender: bytes,
        address: bytes,
        args: typing.List[typing.Any],
        value: int = 0,
        checked: bool = True,
    ) -> types.Transaction:
        """
        Create a transaction to submit to the current simulation block

        Parameters
        ----------
        sender: bytes
            Address of the call sender.
        address: bytes
            Address of the contract to call.
        args: typing.List[typing.Any]
            List of arguments to this function.
        value: int, optional
            Value attached to the transaction, default 0.
        checked: bool, optional
            If ``True`` the simulation will panic if the transaction
            from this call is reverted (default ``False``).

        Returns
        -------
        verbs.types.Transaction
            Transaction that can be submitted to the simulation
            for execution in the next block.
        """
        encoded_args = self.encode(args)
        return (sender, address, encoded_args, value, checked)

    def call(
        self,
        env: types.Env,
        sender: bytes,
        address: bytes,
        args: typing.List[typing.Any],
        value: int = 0,
    ) -> typing.Tuple[typing.Tuple[typing.Any, ...], typing.List, int]:
        """
        Directly call this function without committing any changes

        Parameters
        ----------
        env: verbs.types.Env
            Simulation environment.
        sender: bytes
            Address of the caller.
        address: bytes
            Address of the contract to call.
        args: typing.List[typing.Any]
            List of function arguments.
        value: int, optional
            Value of the transaction.

        Returns
        -------
        results: typing.Tuple[typing.Any, ...]
            Tuple of values returned from the function call.
        logs: typing.List
            List of events/logs generated by this function call.
        gas-used: int
            Gas used by this transaction.
        """
        encoded_args = self.encode(args)
        result, logs, gas = env.call(sender, address, encoded_args, value)
        result = self.decode(result)
        return result, logs, gas

    def execute(
        self,
        env: types.Env,
        sender: bytes,
        address: bytes,
        args: typing.List[typing.Any],
        value: int = 0,
    ) -> typing.Tuple[typing.Tuple[typing.Any, ...], typing.List, int]:
        """
        Directly call this function and commit any changes

        Parameters
        ----------
        env: verbs.types.Env
            Simulation environment.
        sender: bytes
            Address of the caller.
        address: bytes
            Address of the contract to call.
        args: typing.List[typing.Any]
            List of function arguments.
        value: int, optional
            Value of the transaction.

        Returns
        -------
        results: typing.Tuple[typing.Any, ...]
            Tuple of values returned from the function call.
        logs: typing.List
            List of events/logs generated by this function call.
        gas-used: int
            Gas used by this transaction.
        """
        encoded_args = self.encode(args)
        result, logs, gas = env.execute(sender, address, encoded_args, value)
        result = self.decode(result)
        return result, logs, gas


class Event:
    """
    ABI event class

    Class representing an ABI event with functionality
    to decode event byte data.

    Parameters
    ----------
    abi: typing.Dict
        Parsed ABI JSON of the event.

    Attributes
    ----------
    inputs: typing.List[str]
        List of event input types.
    """

    def __init__(self, abi: typing.Dict):
        self.inputs = parse_input_types(abi)

    def decode(self, data: bytes) -> typing.Tuple[typing.Any, ...]:
        """
        Decode data from this event type

        Parameters
        ----------
        data: bytes
            Data from an instance of this event.

        Returns
        -------
        typing.Tuple[typing.Any, ...]
            Tuple of python values parsed from this event type.
        """
        return eth_abi.decode(self.inputs, data)


def get_abi(name: str, abi: typing.List[typing.Dict]) -> type:
    """
    Create an ABI type from ABI JSON

    Create a new ``type`` from parsed ABI JSON with attributes
    for the contracts functions and events and a ``constructor``
    attribute used for contract deployment.

    Parameters
    ----------
    name: str
        Name to give the ABI type.
    abi: typing.List[typing.Dict]
        Parsed ABI JSON.

    Returns
    -------
    type
        Type representing the ABI with attributes for
        the functions and events.
    """
    grouped = dict()

    for a in abi:
        if a["type"] not in ("function", "event", "constructor"):
            continue

        if a["type"] == "constructor":
            nm = "constructor"
        else:
            nm = a["name"]

        if nm not in grouped:
            grouped[nm] = list()

        grouped[nm].append(a)

    for v in grouped.values():
        if len(v) > 1:
            for i in range(len(v)):
                v[i]["name"] = v[i]["name"] + str(i)

    methods = dict()

    for v in grouped.values():
        for a in v:
            if a["type"] == "function":
                methods[a["name"]] = Function(a)
            elif a["type"] == "event":
                methods[a["name"]] = Event(a)

    if "constructor" not in grouped:
        methods["constructor"] = Constructor(name, dict(inputs=[]))
    else:
        methods["constructor"] = Constructor(name, grouped["constructor"][0])

    return type(name, (), methods)


def abi_from_str(name: str, json_str: str) -> type:
    """
    Create an ABI type from a JSON string

    Create a new ABI ``type`` from an ABI JSON string.

    Parameters
    ----------
    name: str
        Name to give the ABI type.
    json_str: str
        ABI JSON string.

    Returns
    -------
    type
        Type representing the ABI with attributes for
        the functions and events.
    """
    abi_json = json.loads(json_str)
    return get_abi(name, abi_json)


def load_abi(abi_path: str) -> type:
    """
    Load an ABI type from an ABI file

    Load an ABI JSON file and create a ``type``
    representing the ABI.

    Parameters
    ----------
    abi_path: str
        Path to the ABI JSON file.

    Returns
    -------
    type
        Type representing the ABI with attributes for
        the functions and events.
    """
    name = abi_path.split("/")[-1].split(".")[0]

    with open(abi_path, "r") as f:
        abi_json = json.load(f)

    return get_abi(name, abi_json)
