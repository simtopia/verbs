import typing

import eth_abi
import eth_utils


def encode_args(
    types: typing.List[str],
    args: typing.List[typing.Any],
) -> typing.List[int]:
    return list(eth_abi.encode(types, args))


def encode_function_args(
    f_abi: typing.Dict,
    types: typing.List[str],
    args: typing.List[typing.Any],
) -> typing.List[int]:

    a = eth_utils.abi.function_abi_to_4byte_selector(f_abi)
    b = eth_abi.encode(types, args)

    return list(a) + list(b)


def hex_to_byte_list(hex: str) -> typing.List[int]:

    if hex.startswith("0x"):
        hex = hex[2:]

    return list(bytes.fromhex(hex))


def int_to_address(i: int):
    return list(i.to_bytes(20, "big"))
