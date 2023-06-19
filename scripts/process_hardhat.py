import json
import sys
from os import listdir
from os.path import isfile, join


def format_arg(a):
    return str(a).replace("'", "").replace("0x", "").replace(" ", "").lower()


def process_deployment_files(path, out_path):
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

            params = dict(
                bytecode=x["bytecode"][2:],
                deploy_address=x["address"][2:],
                constructor_args=args,
                name=fx,
            )

            with open(f"{out_path}/{fx}.json", "w") as params_file:
                json.dump(params, params_file, indent=4)

    with open(f"{out_path}/contract_names.json", "w") as contracts_file:
        json.dump(file_names, contracts_file, indent=4)


if __name__ == "__main__":
    args = sys.argv

    process_deployment_files(args[1], args[2])
