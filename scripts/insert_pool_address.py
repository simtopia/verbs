import json
import sys

ADMIN = "0000000000000000000000000000000000000001"


def insert_pool_adddress_into_tokens(contract_path):
    """
    Use this to insert missing pool addresses in token contract definitions
    """

    with open(f"{contract_path}/contract_names.json") as f:
        contract_list = json.load(f)

    with open(f"{contract_path}/Pool-Proxy-Test.json") as f:
        pool = json.load(f)

    pool_address = pool["deploy_address"]

    for contract in contract_list:

        if any(
            substring in contract
            for substring in ["-AToken", "-StableDebtToken", "-VariableDebtToken"]
        ):

            f_path = f"{contract_path}/{contract}.json"

            with open(f_path, "r") as fp:
                params = json.load(fp)
                params["constructor_args"] = [pool_address]

            with open(f_path, "w") as fp:
                json.dump(params, fp, indent=4)

        if "Proxy" in contract:

            f_path = f"{contract_path}/{contract}.json"

            with open(f_path, "r") as fp:
                params = json.load(fp)
                params["constructor_args"] = [ADMIN]

            with open(f_path, "w") as fp:
                json.dump(params, fp, indent=4)


if __name__ == "__main__":
    args = sys.argv
    insert_pool_adddress_into_tokens(args[1])
