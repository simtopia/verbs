import pytest

from verbs import envs, utils

INITIAL_VALUE = 101


@pytest.fixture
def bytecode():
    hex = (
        "0x608060405234801561001057600080fd5b5060405161026f3803806102"
        "6f8339818101604052810190610032919061007a565b8060008190555050"
        "6100a7565b600080fd5b6000819050919050565b61005781610044565b81"
        "1461006257600080fd5b50565b6000815190506100748161004e565b9291"
        "5050565b6000602082840312156100905761008f61003f565b5b60006100"
        "9e84828501610065565b91505092915050565b6101b9806100b660003960"
        "00f3fe608060405234801561001057600080fd5b50600436106100365760"
        "003560e01c8063209652551461003b5780635093dc7d14610059575b6000"
        "80fd5b610043610075565b60405161005091906100e1565b604051809103"
        "90f35b610073600480360381019061006e919061012d565b61007e565b00"
        "5b60008054905090565b600080549050816000819055507fb8ba75888072"
        "4160775cc09f9aa6f15e3d6be6aed023b548a74a72981f806f6381836040"
        "516100bc92919061015a565b60405180910390a15050565b600081905091"
        "9050565b6100db816100c8565b82525050565b60006020820190506100f6"
        "60008301846100d2565b92915050565b600080fd5b61010a816100c8565b"
        "811461011557600080fd5b50565b60008135905061012781610101565b92"
        "915050565b600060208284031215610143576101426100fc565b5b600061"
        "015184828501610118565b91505092915050565b60006040820190506101"
        "6f60008301856100d2565b61017c60208301846100d2565b939250505056"
        "fea264697066735822122023e51761b7f66dc61568d8bcae2f6f8877a5c3"
        "f186e8ebb9c89260d547e2a8f864736f6c634300080a0033"
    )

    return utils.hex_to_bytes(hex)


@pytest.fixture
def test_abi():
    return [
        {
            "inputs": [{"internalType": "int256", "name": "x", "type": "int256"}],
            "stateMutability": "nonpayable",
            "type": "constructor",
        },
        {
            "anonymous": False,
            "inputs": [
                {
                    "indexed": False,
                    "internalType": "int256",
                    "name": "old_value",
                    "type": "int256",
                },
                {
                    "indexed": False,
                    "internalType": "int256",
                    "name": "new_value",
                    "type": "int256",
                },
            ],
            "name": "ValueUpdated",
            "type": "event",
        },
        {
            "inputs": [],
            "name": "getValue",
            "outputs": [{"internalType": "int256", "name": "", "type": "int256"}],
            "stateMutability": "view",
            "type": "function",
        },
        {
            "inputs": [{"internalType": "int256", "name": "x", "type": "int256"}],
            "name": "setValue",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function",
        },
    ]


@pytest.fixture
def constructor_args():
    return utils.encode_args(["int256"], [INITIAL_VALUE])


@pytest.fixture
def env():
    return envs.EmptyEnv(1234)
