import typing

import eth_abi
import eth_utils


def encode_function_args(
    f_abi: typing.Dict,
    types: typing.List[str],
    args: typing.List[typing.Any],
):
    a = eth_utils.abi.function_abi_to_4byte_selector(f_abi)
    b = eth_abi.encode(types, args)

    return list(a) + list(b)
