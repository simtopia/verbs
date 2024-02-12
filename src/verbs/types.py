"""
Type aliases
"""

import typing

from verbs.envs import EmptyEnv, ForkEnv

Env = typing.Union[EmptyEnv, ForkEnv]

Transaction = typing.Tuple[bytes, bytes, bytes, int, bool]

Cache = typing.Tuple[
    int,
    int,
    typing.List[typing.Tuple[bytes, typing.Tuple[bytes, int, bytes, bytes]]],
    typing.List[typing.Tuple[bytes, bytes, bytes]],
]
