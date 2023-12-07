"""
Data types
"""
from dataclasses import dataclass


@dataclass
class Call:
    """
    Struct of call arguments

    This class wraps field that are submitted
    to be processed by the EVM during a simulation.

    Simulated agents should return a list of
    calls, which are then processed in the next block.

    Attributes
    ----------
    sender: bytes
        Address of the transaction caller/sender.
    contract_address: bytes
        Address of the contract to call.
    encoded_args: bytes
        ABI encoded arguments (with function selector).
    checked: bool
        If ``True`` the simulation will raise an exception
        if the transaction is reverted. This can be used
        to ensure transactions are successful, but should
        be set to ``False`` if a transaction can fail but
        the simulation should continue.
    """

    sender: bytes
    contract_address: bytes
    encoded_args: bytes
    checked: bool
