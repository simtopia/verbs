"""
Type aliases
"""

import typing

from verbs.envs import (
    EmptyEnvGasPriority,
    EmptyEnvRandom,
    ForkEnvGasPriority,
    ForkEnvRandom,
)

Env = typing.Union[
    EmptyEnvRandom, EmptyEnvGasPriority, ForkEnvRandom, ForkEnvGasPriority
]

Transaction = typing.Tuple[
    bytes,
    bytes,
    bytes,
    bool,
    typing.Optional[int],
    typing.Optional[int],
    typing.Optional[int],
]

Cache = typing.Tuple[
    int,
    int,
    typing.List[typing.Tuple[bytes, typing.Tuple[bytes, int, bytes, bytes]]],
    typing.List[typing.Tuple[bytes, bytes, bytes]],
]
