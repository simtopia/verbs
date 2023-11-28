import eth_abi
import pytest

from verbs import EmptyEnv, utils

INITIAL_VALUE = 101


@pytest.fixture
def bytecode():
    hex = (
        "0x608060405234801561001057600080fd5b506040516102063803806102"
        "068339818101604052810190610032919061007a565b8060008190555050"
        "6100a7565b600080fd5b6000819050919050565b61005781610044565b81"
        "1461006257600080fd5b50565b6000815190506100748161004e565b9291"
        "5050565b6000602082840312156100905761008f61003f565b5b60006100"
        "9e84828501610065565b91505092915050565b610150806100b660003960"
        "00f3fe608060405234801561001057600080fd5b50600436106100365760"
        "003560e01c8063209652551461003b5780635093dc7d14610059575b6000"
        "80fd5b610043610075565b60405161005091906100a1565b604051809103"
        "90f35b610073600480360381019061006e91906100ed565b61007e565b00"
        "5b60008054905090565b8060008190555050565b6000819050919050565b"
        "61009b81610088565b82525050565b60006020820190506100b660008301"
        "84610092565b92915050565b600080fd5b6100ca81610088565b81146100"
        "d557600080fd5b50565b6000813590506100e7816100c1565b9291505056"
        "5b600060208284031215610103576101026100bc565b5b60006101118482"
        "85016100d8565b9150509291505056fea2646970667358221220d99fa7a1"
        "1a5739cf9f1c4e30ebbb603943f8e1e44a3b4c0c10c3ea53799a236d6473"
        "6f6c634300080a0033"
    )

    return utils.hex_to_byte_list(hex)


@pytest.fixture
def constructor_args():
    return utils.encode_args(["int256"], [INITIAL_VALUE])


@pytest.fixture
def env():
    return EmptyEnv(1234, "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266")


@pytest.fixture
def contract():

    return [
        {
            "inputs": [{"internalType": "int256", "name": "x", "type": "int256"}],
            "stateMutability": "nonpayable",
            "type": "constructor",
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


def test_deploy_and_call_contract(env, bytecode, constructor_args, contract):

    # Deploy contract and get address
    address = env.deploy_contract("test_contract", bytecode + constructor_args)

    # Get current contract value
    call_0 = utils.encode_function_args(contract[1], [], [])
    result_0 = env.call(env.admin_address, address, call_0, 0)
    result_0 = eth_abi.decode(["int256"], bytes(result_0[0]))
    assert result_0[0] == INITIAL_VALUE

    # Update contract value
    call_1 = utils.encode_function_args(contract[2], ["int256"], [202])
    env.execute(env.admin_address, address, call_1, 0)

    # Get new contract value
    call_2 = utils.encode_function_args(contract[1], [], [])
    result_2 = env.call(env.admin_address, address, call_2, 0)
    result_2 = eth_abi.decode(["int256"], bytes(result_2[0]))
    assert result_2[0] == 202


def test_sim_update(env, bytecode, constructor_args, contract):

    # Deploy contract and get address
    address = env.deploy_contract("test_contract", bytecode + constructor_args)

    # Get current contract value
    call_0 = utils.encode_function_args(contract[1], [], [])
    result_0 = env.call(env.admin_address, address, call_0, 0)
    result_0 = eth_abi.decode(["int256"], bytes(result_0[0]))
    assert result_0[0] == INITIAL_VALUE

    # Submit contract update call
    call_1 = utils.encode_function_args(contract[2], ["int256"], [202])
    env.submit_call(env.admin_address, address, call_1, True)

    env.process_block()

    assert env.step == 1

    # Get new value after block update
    call_2 = utils.encode_function_args(contract[1], [], [])
    result_2 = env.call(env.admin_address, address, call_2, 0)
    result_2 = eth_abi.decode(["int256"], bytes(result_2[0]))
    assert result_2[0] == 202
