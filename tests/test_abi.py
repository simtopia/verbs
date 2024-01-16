from verbs import abi


def test_abi_init(test_abi):
    a = abi.get_abi("Foo", test_abi)

    encoded = a.setValue.encode([10])

    assert isinstance(encoded, bytes)
    assert len(encoded) == 36

    decoded = a.getValue.decode(encoded[4:])

    assert decoded == (10,)

    decoded_event = a.ValueUpdated.decode(encoded[4:] + encoded[4:])

    assert decoded_event == (10, 10)

    inputs = a.ValueUpdated.inputs

    assert inputs == ["int256", "int256"]


def test_abi_from_file():
    a = abi.load_abi("tests/test.abi")

    assert a.__name__ == "test"

    encoded = a.setValue.encode([10])

    assert isinstance(encoded, bytes)
    assert len(encoded) == 36

    decoded = a.getValue.decode(encoded[4:])

    assert decoded == (10,)

    decoded_event = a.ValueUpdated.decode(encoded[4:] + encoded[4:])

    assert decoded_event == (10, 10)


def test_abi_w_repeated_key():
    abi_data = [
        {
            "name": "foo",
            "inputs": [{"internalType": "int256", "name": "a", "type": "int256"}],
            "outputs": [],
            "stateMutability": "view",
            "type": "function",
        },
        {
            "name": "foo",
            "inputs": [
                {"internalType": "int256", "name": "a", "type": "int256"},
                {"internalType": "int256", "name": "b", "type": "int256"},
            ],
            "outputs": [],
            "stateMutability": "view",
            "type": "function",
        },
    ]

    a = abi.get_abi("Foo", abi_data)

    assert hasattr(a, "foo0")
    assert hasattr(a, "foo1")

    encoded_0 = a.foo0.encode([100])
    assert len(encoded_0) == 36

    encoded_1 = a.foo1.encode([101, 202])
    assert len(encoded_1) == 68


def test_abi_from_str():

    abi_str = (
        '[{"name": "foo", "inputs": '
        '[{"internalType": "int256", "name": "a", "type": "int256"}],'
        ' "outputs": [], "stateMutability": "view", "type": "function"}]'
    )

    a = abi.abi_from_str("Foo", abi_str)

    encoded_0 = a.foo.encode([100])
    assert len(encoded_0) == 36


def test_constructor():
    abi_data = [
        {
            "inputs": [{"name": "x", "type": "uint256"}],
            "payable": False,
            "stateMutability": "nonpayable",
            "type": "constructor",
        }
    ]

    a = abi.get_abi("Foo", abi_data)

    assert hasattr(a, "constructor")
    assert a.constructor.inputs == ["uint256"]


def test_default_constructor():
    abi_data = []

    a = abi.get_abi("Foo", abi_data)

    assert hasattr(a, "constructor")
    assert a.constructor.inputs == []


def test_abi_recursive_input():
    a = abi.load_abi("tests/test.abi")
    assert a.setValue.inputs == ["int256"]
    assert a.exactInputSingle.inputs == [
        "(address,address,uint24,address,uint256,uint256,uint256,uint160)"
    ]
    assert a.test_recursive_tuple.inputs == ["(address,address,(address,address))"]
