from verbs import envs, utils


def test_cache_to_json(cache_json):

    decoded = utils.cache_from_json(cache_json)
    json = utils.cache_to_json(decoded)

    assert json == cache_json


def test_env_from_cache(cache_json, cache_abi):

    cache = utils.cache_from_json(cache_json)
    env = envs.EmptyEnvRandom(101, cache=cache)

    contract = utils.hex_to_bytes("0x6B175474E89094C44Da98b954EedeAC495271d0F")

    decimals = cache_abi.decimals.call(env, utils.ZERO_ADDRESS, contract, [])[0][0]

    assert decimals == 18

    name = cache_abi.name.call(env, utils.ZERO_ADDRESS, contract, [])[0][0]

    assert name == "Dai Stablecoin"

    supply = cache_abi.totalSupply.call(env, utils.ZERO_ADDRESS, contract, [])[0][0]

    assert supply == 3649626828757146571328619810
