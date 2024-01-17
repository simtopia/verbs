"""
Ethereum ABI and numerical utilities
"""
import typing

import eth_abi
import eth_utils
from solcx import compile_files, install_solc


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
