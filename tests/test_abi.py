from verbs import abi


def test_abi_init(test_abi):
    a = abi.get_abi("Foo", test_abi)

    encoded = a.setValue.encode([10])

    assert isinstance(encoded, list)
    assert len(encoded) == 36

    decoded = a.getValue.decode(encoded[4:])

    assert decoded == (10,)

    decoded_event = a.ValueUpdated.decode(encoded[4:] + encoded[4:])

    assert decoded_event == (10, 10)
