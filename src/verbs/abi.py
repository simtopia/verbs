import json
import typing

import eth_abi
import eth_utils


class Function:
    def __init__(self, abi: typing.Dict):
        self.inputs = [x["type"] for x in abi["inputs"]]
        self.outputs = [x["type"] for x in abi["outputs"]]
        self.selector = eth_utils.abi.function_abi_to_4byte_selector(abi)

    def encode(self, args: typing.List[typing.Any]) -> typing.List[int]:
        return self.selector + eth_abi.encode(self.inputs, args)

    def decode(self, output: bytes) -> typing.Tuple[typing.Any, ...]:
        return eth_abi.decode(self.outputs, bytes(output))


class Event:
    def __init__(self, abi: typing.Dict):
        self.inputs = [x["type"] for x in abi["inputs"] if not x["indexed"]]

    def decode(self, output: bytes) -> typing.Tuple[typing.Any, ...]:
        return eth_abi.decode(self.inputs, output)


def get_abi(name: str, abi: typing.List[typing.Dict]) -> type:

    grouped = dict()

    for a in abi:
        if a["type"] not in ("function", "event"):
            continue

        nm = a["name"]

        if nm not in grouped:
            grouped[nm] = list()

        grouped[nm].append(a)

    for v in grouped.values():
        if len(v) > 1:
            for i in range(len(v)):
                v[i]["name"] = v[i]["name"] + str(i)

    methods = dict()

    for v in grouped.values():
        for a in v:
            if a["type"] == "function":
                methods[a["name"]] = Function(a)
            elif a["type"] == "event":
                methods[a["name"]] = Event(a)

    return type(name, (), methods)


def abi_from_str(name: str, json_str: str) -> type:
    abi_json = json.loads(json_str)
    return get_abi(name, abi_json)


def load_abi(abi_path: str) -> type:

    name = abi_path.split("/")[-1].split(".")[0]

    with open(abi_path, "r") as f:
        abi_json = json.load(f)

    return get_abi(name, abi_json)
