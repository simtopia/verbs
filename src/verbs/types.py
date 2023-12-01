from dataclasses import dataclass


@dataclass
class Call:
    sender: bytes
    contract_address: bytes
    encoded_args: bytes
    checked: bool
