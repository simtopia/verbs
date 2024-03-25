import numpy as np
import pytest

from verbs import abi, envs, utils

INITIAL_VALUE = 101


@pytest.fixture
def bytecode():
    hex = (
        "0x608060405234801561001057600080fd5b506040516103313803806103"
        "318339818101604052810190610032919061007a565b8060008190555050"
        "6100a7565b600080fd5b6000819050919050565b61005781610044565b81"
        "1461006257600080fd5b50565b6000815190506100748161004e565b9291"
        "5050565b6000602082840312156100905761008f61003f565b5b60006100"
        "9e84828501610065565b91505092915050565b61027b806100b660003960"
        "00f3fe608060405234801561001057600080fd5b50600436106100365760"
        "003560e01c8063209652551461003b5780635093dc7d14610059575b6000"
        "80fd5b610043610075565b6040516100509190610126565b604051809103"
        "90f35b610073600480360381019061006e9190610172565b61007e565b00"
        "5b60008054905090565b6103e88113156100c3576040517f08c379a00000"
        "000000000000000000000000000000000000000000000000000081526004"
        "016100ba906101fc565b60405180910390fd5b6000805490508160008190"
        "55507fb8ba758880724160775cc09f9aa6f15e3d6be6aed023b548a74a72"
        "981f806f63818360405161010192919061021c565b60405180910390a150"
        "50565b6000819050919050565b6101208161010d565b82525050565b6000"
        "60208201905061013b6000830184610117565b92915050565b600080fd5b"
        "61014f8161010d565b811461015a57600080fd5b50565b60008135905061"
        "016c81610146565b92915050565b60006020828403121561018857610187"
        "610141565b5b60006101968482850161015d565b91505092915050565b60"
        "0082825260208201905092915050565b7f2056616c7565206d7573742062"
        "65206c657373207468616e2031303030000000600082015250565b600061"
        "01e6601d8361019f565b91506101f1826101b0565b602082019050919050"
        "565b60006020820190508181036000830152610215816101d9565b905091"
        "9050565b60006040820190506102316000830185610117565b61023e6020"
        "830184610117565b939250505056fea2646970667358221220a3de9113ee"
        "b3f4094e585e8eb6de3c4a72bf5615c5cdba826ea9d63366251ce464736f"
        "6c634300080a0033"
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
                    "indexed": True,
                    "internalType": "address",
                    "name": "by",
                    "type": "address",
                },
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
    return envs.EmptyEnvRandom(1234)


@pytest.fixture
def agent_type():
    class Agent:
        def __init__(self, i: int, contract: bytes, abi, offset: int = 0):
            self.address = utils.int_to_address(i)
            self.contract = contract
            self.abi = abi
            self.current = 0
            self.offset = offset

        def update(
            self,
            rng: np.random.Generator,
            env,
        ):
            self.current = self.abi.getValue.call(env, self.address, self.contract, [])[
                0
            ][0]

            set_call = self.abi.setValue.transaction(
                self.address, self.contract, [self.current + 1]
            )

            return [set_call]

        def record(self, env):
            return self.current + self.offset

    return Agent


@pytest.fixture
def cache_json():
    return [
        1702552271,
        18784000,
        [
            [
                "0000000000000000000000000000000000000000",
                [
                    "2486033169d32126d30200000000000000000000000000000000000000000000",
                    0,
                    "c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470",
                    (
                        "000000000000000000000000000000000"
                        "000000000000000000000000000000000"
                    ),
                ],
            ],
            [
                "6b175474e89094c44da98b954eedeac495271d0f",
                [
                    "0000000000000000000000000000000000000000000000000000000000000000",
                    1,
                    "4e36f96ee1667a663dfaac57c4d185a0e369a3a217e0079d49620f34f85d1ac7",
                    (
                        "608060405234801561001057600080fd5b50600436106101425760003560e"
                        "01c80637ecebe00116100b8578063a9059cbb1161007c578063a9059cbb14"
                        "6106b4578063b753a98c1461071a578063bb35783b14610768578063bf353"
                        "dbb146107d6578063dd62ed3e1461082e578063f2d5d56b146108a6576101"
                        "42565b80637ecebe00146104a15780638fcbaf0c146104f957806395d89b4"
                        "11461059f5780639c52a7f1146106225780639dc29fac1461066657610142"
                        "565b8063313ce5671161010a578063313ce567146102f25780633644e5151"
                        "461031657806340c10f191461033457806354fd4d501461038257806365fa"
                        "e35e1461040557806370a082311461044957610142565b806306fdde03146"
                        "10147578063095ea7b3146101ca57806318160ddd1461023057806323b872"
                        "dd1461024e57806330adf81f146102d4575b600080fd5b61014f6108f4565"
                        "b604051808060200182810382528381815181526020019150805190602001"
                        "9080838360005b8381101561018f578082015181840152602081019050610"
                        "174565b50505050905090810190601f1680156101bc578082038051600183"
                        "6020036101000a031916815260200191505b509250505060405180910390f"
                        "35b610216600480360360408110156101e057600080fd5b81019080803573"
                        "ffffffffffffffffffffffffffffffffffffffff169060200190929190803"
                        "5906020019092919050505061092d565b6040518082151515158152602001"
                        "91505060405180910390f35b610238610a1f565b604051808281526020019"
                        "1505060405180910390f35b6102ba60048036036060811015610264576000"
                        "80fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff1"
                        "69060200190929190803573ffffffffffffffffffffffffffffffffffffff"
                        "ff16906020019092919080359060200190929190505050610a25565b60405"
                        "1808215151515815260200191505060405180910390f35b6102dc610f3a56"
                        "5b6040518082815260200191505060405180910390f35b6102fa610f61565"
                        "b604051808260ff1660ff16815260200191505060405180910390f35b6103"
                        "1e610f66565b6040518082815260200191505060405180910390f35b61038"
                        "06004803603604081101561034a57600080fd5b81019080803573ffffffff"
                        "ffffffffffffffffffffffffffffffff16906020019092919080359060200"
                        "190929190505050610f6c565b005b61038a611128565b6040518080602001"
                        "828103825283818151815260200191508051906020019080838360005b838"
                        "110156103ca5780820151818401526020810190506103af565b5050505090"
                        "5090810190601f1680156103f75780820380516001836020036101000a031"
                        "916815260200191505b509250505060405180910390f35b61044760048036"
                        "03602081101561041b57600080fd5b81019080803573fffffffffffffffff"
                        "fffffffffffffffffffffff169060200190929190505050611161565b005b"
                        "61048b6004803603602081101561045f57600080fd5b81019080803573fff"
                        "fffffffffffffffffffffffffffffffffffff169060200190929190505050"
                        "61128f565b6040518082815260200191505060405180910390f35b6104e36"
                        "00480360360208110156104b757600080fd5b81019080803573ffffffffff"
                        "ffffffffffffffffffffffffffffff1690602001909291905050506112a75"
                        "65b6040518082815260200191505060405180910390f35b61059d60048036"
                        "0361010081101561051057600080fd5b81019080803573fffffffffffffff"
                        "fffffffffffffffffffffffff169060200190929190803573ffffffffffff"
                        "ffffffffffffffffffffffffffff169060200190929190803590602001909"
                        "2919080359060200190929190803515159060200190929190803560ff1690"
                        "6020019092919080359060200190929190803590602001909291905050506"
                        "112bf565b005b6105a76117fa565b60405180806020018281038252838181"
                        "51815260200191508051906020019080838360005b838110156105e757808"
                        "20151818401526020810190506105cc565b50505050905090810190601f16"
                        "80156106145780820380516001836020036101000a0319168152602001915"
                        "05b509250505060405180910390f35b610664600480360360208110156106"
                        "3857600080fd5b81019080803573fffffffffffffffffffffffffffffffff"
                        "fffffff169060200190929190505050611833565b005b6106b26004803603"
                        "604081101561067c57600080fd5b81019080803573fffffffffffffffffff"
                        "fffffffffffffffffffff1690602001909291908035906020019092919050"
                        "5050611961565b005b610700600480360360408110156106ca57600080fd5"
                        "b81019080803573ffffffffffffffffffffffffffffffffffffffff169060"
                        "20019092919080359060200190929190505050611df4565b6040518082151"
                        "51515815260200191505060405180910390f35b6107666004803603604081"
                        "101561073057600080fd5b81019080803573fffffffffffffffffffffffff"
                        "fffffffffffffff1690602001909291908035906020019092919050505061"
                        "1e09565b005b6107d46004803603606081101561077e57600080fd5b81019"
                        "080803573ffffffffffffffffffffffffffffffffffffffff169060200190"
                        "929190803573ffffffffffffffffffffffffffffffffffffffff169060200"
                        "19092919080359060200190929190505050611e19565b005b610818600480"
                        "360360208110156107ec57600080fd5b81019080803573fffffffffffffff"
                        "fffffffffffffffffffffffff169060200190929190505050611e2a565b60"
                        "40518082815260200191505060405180910390f35b6108906004803603604"
                        "081101561084457600080fd5b81019080803573ffffffffffffffffffffff"
                        "ffffffffffffffffff169060200190929190803573fffffffffffffffffff"
                        "fffffffffffffffffffff169060200190929190505050611e42565b604051"
                        "8082815260200191505060405180910390f35b6108f260048036036040811"
                        "0156108bc57600080fd5b81019080803573ffffffffffffffffffffffffff"
                        "ffffffffffffff16906020019092919080359060200190929190505050611"
                        "e67565b005b6040518060400160405280600e81526020017f446169205374"
                        "61626c65636f696e000000000000000000000000000000000000815250815"
                        "65b600081600360003373ffffffffffffffffffffffffffffffffffffffff"
                        "1673ffffffffffffffffffffffffffffffffffffffff16815260200190815"
                        "260200160002060008573ffffffffffffffffffffffffffffffffffffffff"
                        "1673ffffffffffffffffffffffffffffffffffffffff16815260200190815"
                        "2602001600020819055508273ffffffffffffffffffffffffffffffffffff"
                        "ffff163373ffffffffffffffffffffffffffffffffffffffff167f8c5be1e"
                        "5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b9258460"
                        "40518082815260200191505060405180910390a36001905092915050565b6"
                        "0015481565b600081600260008673ffffffffffffffffffffffffffffffff"
                        "ffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602"
                        "001908152602001600020541015610adc576040517f08c379a00000000000"
                        "0000000000000000000000000000000000000000000000815260040180806"
                        "02001828103825260188152602001807f4461692f696e7375666669636965"
                        "6e742d62616c616e636500000000000000008152506020019150506040518"
                        "0910390fd5b3373ffffffffffffffffffffffffffffffffffffffff168473"
                        "ffffffffffffffffffffffffffffffffffffffff1614158015610bb457507"
                        "fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                        "ffff600360008673ffffffffffffffffffffffffffffffffffffffff1673f"
                        "fffffffffffffffffffffffffffffffffffffff1681526020019081526020"
                        "0160002060003373ffffffffffffffffffffffffffffffffffffffff1673f"
                        "fffffffffffffffffffffffffffffffffffffff1681526020019081526020"
                        "016000205414155b15610db25781600360008673fffffffffffffffffffff"
                        "fffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffff"
                        "ff16815260200190815260200160002060003373fffffffffffffffffffff"
                        "fffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffff"
                        "ff168152602001908152602001600020541015610cab576040517f08c379a"
                        "0000000000000000000000000000000000000000000000000000000008152"
                        "60040180806020018281038252601a8152602001807f4461692f696e73756"
                        "666696369656e742d616c6c6f77616e636500000000000081525060200191"
                        "505060405180910390fd5b610d31600360008673fffffffffffffffffffff"
                        "fffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffff"
                        "ff16815260200190815260200160002060003373fffffffffffffffffffff"
                        "fffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffff"
                        "ff1681526020019081526020016000205483611e77565b600360008673fff"
                        "fffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffff"
                        "ffffffffffffffffffff16815260200190815260200160002060003373fff"
                        "fffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffff"
                        "ffffffffffffffffffff168152602001908152602001600020819055505b6"
                        "10dfb600260008673ffffffffffffffffffffffffffffffffffffffff1673"
                        "ffffffffffffffffffffffffffffffffffffffff168152602001908152602"
                        "0016000205483611e77565b600260008673ffffffffffffffffffffffffff"
                        "ffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168"
                        "15260200190815260200160002081905550610e87600260008573ffffffff"
                        "ffffffffffffffffffffffffffffffff1673fffffffffffffffffffffffff"
                        "fffffffffffffff1681526020019081526020016000205483611e91565b60"
                        "0260008573ffffffffffffffffffffffffffffffffffffffff1673fffffff"
                        "fffffffffffffffffffffffffffffffff1681526020019081526020016000"
                        "20819055508273ffffffffffffffffffffffffffffffffffffffff168473f"
                        "fffffffffffffffffffffffffffffffffffffff167fddf252ad1be2c89b69"
                        "c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef846040518082815"
                        "260200191505060405180910390a3600190509392505050565b7fea2aa0a1"
                        "be11a07ed86d755c93467f4f82362b452371d1ba94d1715123511acb60001"
                        "b81565b601281565b60055481565b60016000803373ffffffffffffffffff"
                        "ffffffffffffffffffffff1673fffffffffffffffffffffffffffffffffff"
                        "fffff1681526020019081526020016000205414611020576040517f08c379"
                        "a000000000000000000000000000000000000000000000000000000000815"
                        "26004018080602001828103825260128152602001807f4461692f6e6f742d"
                        "617574686f72697a656400000000000000000000000000008152506020019"
                        "1505060405180910390fd5b611069600260008473ffffffffffffffffffff"
                        "ffffffffffffffffffff1673fffffffffffffffffffffffffffffffffffff"
                        "fff1681526020019081526020016000205482611e91565b600260008473ff"
                        "ffffffffffffffffffffffffffffffffffffff1673fffffffffffffffffff"
                        "fffffffffffffffffffff1681526020019081526020016000208190555061"
                        "10b860015482611e91565b6001819055508173fffffffffffffffffffffff"
                        "fffffffffffffffff16600073ffffffffffffffffffffffffffffffffffff"
                        "ffff167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55"
                        "a4df523b3ef836040518082815260200191505060405180910390a3505056"
                        "5b6040518060400160405280600181526020017f310000000000000000000"
                        "000000000000000000000000000000000000000000081525081565b600160"
                        "00803373ffffffffffffffffffffffffffffffffffffffff1673fffffffff"
                        "fffffffffffffffffffffffffffffff168152602001908152602001600020"
                        "5414611215576040517f08c379a0000000000000000000000000000000000"
                        "0000000000000000000000081526004018080602001828103825260128152"
                        "602001807f4461692f6e6f742d617574686f72697a6564000000000000000"
                        "000000000000081525060200191505060405180910390fd5b600160008083"
                        "73ffffffffffffffffffffffffffffffffffffffff1673fffffffffffffff"
                        "fffffffffffffffffffffffff168152602001908152602001600020819055"
                        "505961012081016040526020815260e0602082015260e0600060408301376"
                        "024356004353360003560e01c60e01b61012085a45050565b600260205280"
                        "60005260406000206000915090505481565b6004602052806000526040600"
                        "0206000915090505481565b60006005547fea2aa0a1be11a07ed86d755c93"
                        "467f4f82362b452371d1ba94d1715123511acb60001b8a8a8a8a8a6040516"
                        "02001808781526020018673ffffffffffffffffffffffffffffffffffffff"
                        "ff1673ffffffffffffffffffffffffffffffffffffffff168152602001857"
                        "3ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffff"
                        "ffffffffffffffffffffffff1681526020018481526020018381526020018"
                        "2151515158152602001965050505050505060405160208183030381529060"
                        "40528051906020012060405160200180807f1901000000000000000000000"
                        "0000000000000000000000000000000000000008152506002018381526020"
                        "0182815260200192505050604051602081830303815290604052805190602"
                        "001209050600073ffffffffffffffffffffffffffffffffffffffff168973"
                        "ffffffffffffffffffffffffffffffffffffffff16141561148c576040517"
                        "f08c379a00000000000000000000000000000000000000000000000000000"
                        "000081526004018080602001828103825260158152602001807f4461692f6"
                        "96e76616c69642d616464726573732d300000000000000000000000815250"
                        "60200191505060405180910390fd5b6001818585856040516000815260200"
                        "1604052604051808581526020018460ff1660ff1681526020018381526020"
                        "018281526020019450505050506020604051602081039080840390855afa1"
                        "580156114e9573d6000803e3d6000fd5b5050506020604051035173ffffff"
                        "ffffffffffffffffffffffffffffffffff168973fffffffffffffffffffff"
                        "fffffffffffffffffff1614611593576040517f08c379a000000000000000"
                        "0000000000000000000000000000000000000000008152600401808060200"
                        "1828103825260128152602001807f4461692f696e76616c69642d7065726d"
                        "6974000000000000000000000000000081525060200191505060405180910"
                        "390fd5b60008614806115a25750854211155b611614576040517f08c379a0"
                        "0000000000000000000000000000000000000000000000000000000081526"
                        "004018080602001828103825260128152602001807f4461692f7065726d69"
                        "742d657870697265640000000000000000000000000000815250602001915"
                        "05060405180910390fd5b600460008a73ffffffffffffffffffffffffffff"
                        "ffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815"
                        "2602001908152602001600020600081548092919060010191905055871461"
                        "16d6576040517f08c379a0000000000000000000000000000000000000000"
                        "0000000000000000081526004018080602001828103825260118152602001"
                        "807f4461692f696e76616c69642d6e6f6e636500000000000000000000000"
                        "000000081525060200191505060405180910390fd5b6000856116e4576000"
                        "611706565b7ffffffffffffffffffffffffffffffffffffffffffffffffff"
                        "fffffffffffffff5b905080600360008c73ffffffffffffffffffffffffff"
                        "ffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168"
                        "15260200190815260200160002060008b73ffffffffffffffffffffffffff"
                        "ffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168"
                        "152602001908152602001600020819055508873ffffffffffffffffffffff"
                        "ffffffffffffffffff168a73fffffffffffffffffffffffffffffffffffff"
                        "fff167f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200a"
                        "c8c7c3b925836040518082815260200191505060405180910390a35050505"
                        "0505050505050565b6040518060400160405280600381526020017f444149"
                        "0000000000000000000000000000000000000000000000000000000000815"
                        "25081565b60016000803373ffffffffffffffffffffffffffffffffffffff"
                        "ff1673ffffffffffffffffffffffffffffffffffffffff168152602001908"
                        "15260200160002054146118e7576040517f08c379a0000000000000000000"
                        "0000000000000000000000000000000000000081526004018080602001828"
                        "103825260128152602001807f4461692f6e6f742d617574686f72697a6564"
                        "000000000000000000000000000081525060200191505060405180910390f"
                        "d5b60008060008373ffffffffffffffffffffffffffffffffffffffff1673"
                        "ffffffffffffffffffffffffffffffffffffffff168152602001908152602"
                        "001600020819055505961012081016040526020815260e0602082015260e0"
                        "600060408301376024356004353360003560e01c60e01b61012085a450505"
                        "65b80600260008473ffffffffffffffffffffffffffffffffffffffff1673"
                        "ffffffffffffffffffffffffffffffffffffffff168152602001908152602"
                        "001600020541015611a16576040517f08c379a00000000000000000000000"
                        "0000000000000000000000000000000000815260040180806020018281038"
                        "25260188152602001807f4461692f696e73756666696369656e742d62616c"
                        "616e6365000000000000000081525060200191505060405180910390fd5b3"
                        "373ffffffffffffffffffffffffffffffffffffffff168273ffffffffffff"
                        "ffffffffffffffffffffffffffff1614158015611aee57507ffffffffffff"
                        "fffffffffffffffffffffffffffffffffffffffffffffffffffff60036000"
                        "8473ffffffffffffffffffffffffffffffffffffffff1673fffffffffffff"
                        "fffffffffffffffffffffffffff1681526020019081526020016000206000"
                        "3373ffffffffffffffffffffffffffffffffffffffff1673fffffffffffff"
                        "fffffffffffffffffffffffffff1681526020019081526020016000205414"
                        "155b15611cec5780600360008473fffffffffffffffffffffffffffffffff"
                        "fffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020"
                        "0190815260200160002060003373fffffffffffffffffffffffffffffffff"
                        "fffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020"
                        "01908152602001600020541015611be5576040517f08c379a000000000000"
                        "0000000000000000000000000000000000000000000008152600401808060"
                        "20018281038252601a8152602001807f4461692f696e73756666696369656"
                        "e742d616c6c6f77616e636500000000000081525060200191505060405180"
                        "910390fd5b611c6b600360008473fffffffffffffffffffffffffffffffff"
                        "fffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020"
                        "0190815260200160002060003373fffffffffffffffffffffffffffffffff"
                        "fffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020"
                        "019081526020016000205482611e77565b600360008473fffffffffffffff"
                        "fffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffff"
                        "ffffffff16815260200190815260200160002060003373fffffffffffffff"
                        "fffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffff"
                        "ffffffff168152602001908152602001600020819055505b611d356002600"
                        "08473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffff"
                        "ffffffffffffffffffffffffffff168152602001908152602001600020548"
                        "2611e77565b600260008473ffffffffffffffffffffffffffffffffffffff"
                        "ff1673ffffffffffffffffffffffffffffffffffffffff168152602001908"
                        "15260200160002081905550611d8460015482611e77565b60018190555060"
                        "0073ffffffffffffffffffffffffffffffffffffffff168273fffffffffff"
                        "fffffffffffffffffffffffffffff167fddf252ad1be2c89b69c2b068fc37"
                        "8daa952ba7f163c4a11628f55a4df523b3ef8360405180828152602001915"
                        "05060405180910390a35050565b6000611e01338484610a25565b90509291"
                        "5050565b611e14338383610a25565b505050565b611e24838383610a25565"
                        "b50505050565b60006020528060005260406000206000915090505481565b"
                        "6003602052816000526040600020602052806000526040600020600091509"
                        "150505481565b611e72823383610a25565b505050565b6000828284039150"
                        "811115611e8b57600080fd5b92915050565b6000828284019150811015611"
                        "ea557600080fd5b9291505056fea265627a7a72315820c0ae2c29860c0a59"
                        "d5586a579abbcddfe4bcef0524a87301425cbc58c3e94e3164736f6c63430"
                        "0050c00320000000000000000000000000000000000000000000000000000"
                        "00000000000000"
                    ),
                ],
            ],
        ],
        [
            [
                "6b175474e89094c44da98b954eedeac495271d0f",
                "0100000000000000000000000000000000000000000000000000000000000000",
                "2285a236dbce32d38ae6ca0b0000000000000000000000000000000000000000",
            ]
        ],
    ]


@pytest.fixture
def cache_abi():
    abi_json = [
        {
            "constant": True,
            "inputs": [],
            "name": "decimals",
            "outputs": [{"internalType": "uint8", "name": "", "type": "uint8"}],
            "payable": False,
            "stateMutability": "view",
            "type": "function",
        },
        {
            "constant": True,
            "inputs": [],
            "name": "name",
            "outputs": [{"internalType": "string", "name": "", "type": "string"}],
            "payable": False,
            "stateMutability": "view",
            "type": "function",
        },
        {
            "constant": True,
            "inputs": [],
            "name": "totalSupply",
            "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
            "payable": False,
            "stateMutability": "view",
            "type": "function",
        },
    ]

    return abi.get_abi("DAI", abi_json)
