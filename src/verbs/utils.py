"""
Ethereum ABI and numerical utilities
"""

import typing

import eth_abi
import eth_utils
import pandas as pd
from solcx import compile_files, install_solc

import verbs.types


def encode_args(
    types: typing.List[str],
    args: typing.List[typing.Any],
) -> bytes:
    """
    ABI encode arguments

    Parameters
    ----------
    types: typing.List[str]
        List of solidity types
    args: typing.List[typing.Any]
        List of Python arguments

    Returns
    -------
    bytes
        ABI encoded arguments
    """
    return eth_abi.encode(types, args)


def encode_function_args(
    f_abi: typing.Dict,
    types: typing.List[str],
    args: typing.List[typing.Any],
) -> bytes:
    """
    Encode function arguments and prepend the function selector

    Parameters
    ----------
    f_abi: typing.Dict
        Parsed function ABI JSON.
    types: typing.List[str]
        List of solidity types.
    args: typing.List[typing.Any]
        List of Python arguments.

    Returns
    -------
    bytes
        ABI encoded selector and arguments
    """
    a = eth_utils.abi.function_abi_to_4byte_selector(f_abi)
    b = eth_abi.encode(types, args)

    return a + b


def hex_to_bytes(hex: str) -> bytes:
    """
    Convert a hex string to bytes

    Parameters
    ----------
    hex: str
        Hex string (with or without `0x` prefix)

    Returns
    -------
    bytes
        Byte encoding of the hex string
    """
    if hex.startswith("0x"):
        hex = hex[2:]

    return bytes.fromhex(hex)


def int_to_address(i: int) -> bytes:
    """
    Convert an integer into address bytes

    Parameters
    ----------
    i: int
        Integer to convert.

    Returns
    -------
    bytes
        Byte encoded address.

    """
    return i.to_bytes(20, "big")


ZERO_ADDRESS = int_to_address(0)


def process_contract(contract_path: str, solc_version: str) -> typing.List[typing.Dict]:
    """
    Compile a solidity contract and return its abi and bytecode

    Parameters
    ----------
    contract_path: str
        Path to solidity contract.
    solc_version: str
        Solidity version used for compilation.

    Returns
    -------
    typing.List[typing.Dict]
        List of dictionary containing the contract name,
        abi JSON and bytecode hex string.
    """
    install_solc(version=solc_version)

    compiled_sol = compile_files(
        [contract_path],
        output_values=["abi", "bin"],
        solc_version=solc_version,
        allow_paths="../",
    )

    return [
        dict(name=k.split(":")[-1], abi=v["abi"], bin=v["bin"])
        for k, v in compiled_sol.items()
    ]


def cache_to_json(cache: verbs.types.Cache) -> typing.List:
    """
    Convert a request cache to a JSON compatible data-structure

    Converts a cache of data requests from a fork-env (generated
    using :py:meth:`verbs.envs.ForkEnv.export_cache`) into
    a JSON friendly data structure, i.e. it converts tuples to
    lists and bytes to hex strings.

    Examples
    --------

    .. code-block:: python

       env = verbs.envs.ForkEnv(...)
       cache = env.export_cache()
       cache_json = verbs.utils.cache_to_json(cache)
       # Can now export the cache to a JSON file

    Parameters
    ----------
    cache: verbs.types.Cache
        Cache generated using :py:meth:`verbs.envs.ForkEnv.export_cache`.

    Returns
    -------
    typing.List
        JSON compatible data structure with bytes replaces with
        corresponding hex string.
    """
    accounts = [
        [x[0].hex(), [x[1][0].hex(), x[1][1], x[1][2].hex(), x[1][3].hex()]]
        for x in cache[2]
    ]
    storage = [[x[0].hex(), x[1].hex(), x[2].hex()] for x in cache[3]]
    return [cache[0], cache[1], accounts, storage]


def cache_from_json(cache_json: typing.List) -> verbs.types.Cache:
    """
    Convert a cache JSON data into env compatible format

    Converts a JSON compatible data structure representing a cache
    back into the format required for use when initialising a
    simulation environment.

    Examples
    --------

    .. code-block:: python

       cache =verbs.utils.cache_from_json(cache_json)
       env = verbs.envs.EmptyEnv(101, cache)

    Parameters
    ----------
    cache_json: typing.List
        Cache JSON data structure.

    Returns
    -------
    verbs.types.Cache
        Cache converted to format for initialisation of a
        simulation environment.
    """
    accounts = [
        (
            bytes.fromhex(x[0]),
            (
                bytes.fromhex(x[1][0]),
                x[1][1],
                bytes.fromhex(x[1][2]),
                bytes.fromhex(x[1][3]),
            ),
        )
        for x in cache_json[2]
    ]
    storage = [
        (bytes.fromhex(x[0]), bytes.fromhex(x[1]), bytes.fromhex(x[2]))
        for x in cache_json[3]
    ]
    return (cache_json[0], cache_json[1], accounts, storage)


def events_to_dataframe(events: typing.List[typing.Tuple]) -> pd.DataFrame:
    """
    Convert a list of event tuples to a Pandas dataframe

    Convert a list of events returned from
    :py:meth:`verbs.envs.EmptyEnv.get_event_history` to
    a Pandas dataframe. Note that brevity the dataframe
    omits the logs attached to the events.

    Parameters
    ----------
    events: list
        List of tuples representing simulation events.

    Returns
    -------
    pd.DataFrame
        Dataframe containing records of simulation events
    """

    columns = ["success", "selector", "step", "sequence"]

    df = pd.DataFrame.from_records(
        [(x[0], x[1], x[3], x[4]) for x in events], columns=columns
    )

    return df
