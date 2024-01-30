"""
Type aliases
"""
import typing

from verbs.envs import EmptyEnv, ForkEnv

Env = typing.Union[EmptyEnv, ForkEnv]

Transaction = typing.Tuple[bytes, bytes, bytes, int, bool]
