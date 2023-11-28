import pytest


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
