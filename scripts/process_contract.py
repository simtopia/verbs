import json
import sys

from solcx import compile_files, install_solc


def process_contract(contract_path, solc_version, output_path):

    install_solc(version=solc_version)

    compiled_sol = compile_files(
        [contract_path],
        output_values=["abi", "bin"],
        solc_version=solc_version,
        allow_paths="../",
    )

    for k, v in compiled_sol.items():

        contract_name = k.split(":")[1]

        abi = v["abi"]
        bin = v["bin"]

        if bin and abi:

            if not bin[:2] == "0x":
                bin = "0x" + bin

            with open(f"{output_path}/{contract_name}.abi", "w") as f:
                json.dump(abi, f, indent=4)

            deploy_address = "0x0000000000000000000000000000000000000000"

            params = dict(
                bytecode=bin,
                constructor_args=[],
                deploy_address=deploy_address,
                name=contract_name,
            )

            with open(f"{output_path}/{contract_name}.json", "w") as f:
                json.dump(params, f, indent=4)


if __name__ == "__main__":

    args = sys.argv

    process_contract(args[1], args[2], args[3])
