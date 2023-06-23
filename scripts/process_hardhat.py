import json
import sys
from os import listdir
from os.path import isfile, join

import requests


def get_storage_values(address, slot, node_url):
    payload = {
        "method": "eth_getStorageAt",
        "params": [address, slot],
        "jsonrpc": "2.0",
        "id": 0,
    }

    return requests.post(node_url, json=payload).json()


def format_arg(a):
    return str(a).replace("'", "").replace("0x", "").replace(" ", "").lower()


def process_deployment_files(
    path, out_path, node_url="http://127.0.0.1:8545", get_storage=True
):

    files = [f for f in listdir(path) if isfile(join(path, f)) and f[0] != "."]

    file_names = list()

    for file in files:

        fx = file.split(".")[0]
        file_names.append(fx)

        with open(f"{path}/{file}") as f:
            x = json.load(f)

            abi = x["abi"]

            with open(f"{out_path}/{fx}.abi", "w") as abi_file:
                json.dump(abi, abi_file, indent=4)

            args = x["args"] if "args" in x else []
            args = [format_arg(a) for a in args]

            address = x["address"]

            params = dict(
                bytecode=x["bytecode"][2:],
                deploy_address=address[2:],
                constructor_args=args,
                name=fx,
            )

            if get_storage:
                storage_layout = x["storageLayout"]["storage"]
                slots = [y["slot"] for y in storage_layout]
                params["storage"] = {
                    int(s): get_storage_values(address, s, node_url=node_url)["result"]
                    for s in slots
                }
            else:
                params["storage"] = dict()

            with open(f"{out_path}/{fx}.json", "w") as params_file:
                json.dump(params, params_file, indent=4)

    with open(f"{out_path}/contract_names.json", "w") as contracts_file:
        json.dump(file_names, contracts_file, indent=4)


if __name__ == "__main__":
    args = sys.argv

    process_deployment_files(args[1], args[2])
