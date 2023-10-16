use alloy_sol_types::sol;

sol!(
    ABI,
    r#"[
        {
            "constant": true,
            "inputs": [],
            "name": "name",
            "outputs": [{"name": "", "type": "string"}],
            "payable": false,
            "stateMutability": "view",
            "type": "function"
        },
        {
            "constant": false,
            "inputs": [{"name": "spender", "type": "address"}, {"name": "tokens", "type": "uint256"}],
            "name": "approve",
            "outputs": [{"name": "success", "type": "bool"}],
            "payable": false,
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "constant": true,
            "inputs": [],
            "name": "totalSupply",
            "outputs": [{"name": "", "type": "uint256"}],
            "payable": false,
            "stateMutability": "view",
            "type": "function"
        },
        {
            "constant": false,
            "inputs": [{"name": "from", "type": "address"},
            {"name": "to", "type": "address"},
            {"name": "tokens", "type": "uint256"}],
            "name": "transferFrom",
            "outputs": [{"name": "success", "type": "bool"}],
            "payable": false,
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "constant": true,
            "inputs": [],
            "name": "decimals",
            "outputs": [{"name": "", "type": "uint8"}],
            "payable": false,
            "stateMutability": "view",
            "type": "function"
        },
        {
            "constant": true,
            "inputs": [],
            "name": "_totalSupply",
            "outputs": [{"name": "", "type": "uint256"}],
            "payable": false,
            "stateMutability": "view",
            "type": "function"
        },
        {
            "constant": true,
            "inputs": [{"name": "tokenOwner", "type": "address"}],
            "name": "balanceOf",
            "outputs": [{"name": "balance", "type": "uint256"}],
            "payable": false,
            "stateMutability": "view",
            "type": "function"
        },
        {
            "constant": true,
            "inputs": [{"name": "a", "type": "uint256"},
                {"name": "b", "type": "uint256"}],
            "name": "safeSub",
            "outputs": [{"name": "c", "type": "uint256"}],
            "payable": false,
            "stateMutability": "pure",
            "type": "function"
        },
        {
            "constant": false,
            "inputs": [{"name": "to", "type": "address"},
                {"name": "tokens", "type": "uint256"}],
            "name": "transfer",
            "outputs": [{"name": "success", "type": "bool"}],
            "payable": false,
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "constant": true,
            "inputs": [{"name": "a", "type": "uint256"},
                {"name": "b", "type": "uint256"}],
            "name": "safeDiv",
            "outputs": [{"name": "c", "type": "uint256"}],
            "payable": false,
            "stateMutability": "pure",
            "type": "function"},
        {
            "constant": false,
            "inputs": [{"name": "spender", "type": "address"},
                {"name": "tokens", "type": "uint256"},
                {"name": "data", "type": "bytes"}],
            "name": "approveAndCall",
            "outputs": [{"name": "success", "type": "bool"}],
            "payable": false,
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "constant": true,
            "inputs": [{"name": "a", "type": "uint256"},
                {"name": "b", "type": "uint256"}],
            "name": "safeMul",
            "outputs": [{"name": "c", "type": "uint256"}],
            "payable": false,
            "stateMutability": "pure",
            "type": "function"
        },
        {
            "constant": true,
            "inputs": [{"name": "tokenOwner", "type": "address"},
                {"name": "spender", "type": "address"}],
            "name": "allowance",
            "outputs": [{"name": "remaining", "type": "uint256"}],
            "payable": false,
            "stateMutability": "view",
            "type": "function"
        },
        {
            "constant": true,
            "inputs": [{"name": "a", "type": "uint256"},
                {"name": "b", "type": "uint256"}],
            "name": "safeAdd",
            "outputs": [{"name": "c", "type": "uint256"}],
            "payable": false,
            "stateMutability": "pure",
            "type": "function"
        },
        {
            "inputs": [{"name": "total_supply", "type": "uint256"}],
            "payable": false,
            "stateMutability": "nonpayable",
            "type": "constructor"},
            {"payable": true, "stateMutability": "payable", "type": "fallback"},
            {"anonymous": false,
            "inputs": [{"indexed": true, "name": "from", "type": "address"},
                {"indexed": true, "name": "to", "type": "address"},
                {"indexed": false, "name": "tokens", "type": "uint256"}],
            "name": "Transfer",
            "type": "event"
        },
        {
            "anonymous": false,
            "inputs": [{"indexed": true, "name": "tokenOwner", "type": "address"},
                {"indexed": true, "name": "spender", "type": "address"},
                {"indexed": false, "name": "tokens", "type": "uint256"}],
            "name": "Approval",
            "type": "event"
        }
    ]
    "#
);

pub const BYTECODE: &str = "608060405234801561001057600080fd5b506040516020806111\
cf833981018060405281019080805190602001909291905050506040805190810160405280600581\
526020017f546f6b656e000000000000000000000000000000000000000000000000000000815250\
6000908051906020019061007e929190610156565b506002600160006101000a81548160ff021916\
908360ff16021790555080600281905550600254600360003373ffffffffffffffffffffffffffff\
ffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001\
600020819055503373ffffffffffffffffffffffffffffffffffffffff16600073ffffffffffffff\
ffffffffffffffffffffffffff167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628\
f55a4df523b3ef6002546040518082815260200191505060405180910390a3506101fb565b828054\
600181600116156101000203166002900490600052602060002090601f016020900481019282601f\
1061019757805160ff19168380011785556101c5565b828001600101855582156101c5579182015b\
828111156101c45782518255916020019190600101906101a9565b5b5090506101d291906101d656\
5b5090565b6101f891905b808211156101f45760008160009055506001016101dc565b5090565b90\
565b610fc58061020a6000396000f3006080604052600436106100d0576000357c01000000000000\
00000000000000000000000000000000000000000000900463ffffffff16806306fdde03146100d5\
578063095ea7b31461016557806318160ddd146101ca57806323b872dd146101f5578063313ce567\
1461027a5780633eaaf86b146102ab57806370a08231146102d6578063a293d1e81461032d578063\
a9059cbb14610378578063b5931f7c146103dd578063cae9ca5114610428578063d05c78da146104\
d3578063dd62ed3e1461051e578063e6cb901314610595575b600080fd5b3480156100e157600080\
fd5b506100ea6105e0565b6040518080602001828103825283818151815260200191508051906020\
019080838360005b8381101561012a57808201518184015260208101905061010f565b5050505090\
5090810190601f1680156101575780820380516001836020036101000a031916815260200191505b\
509250505060405180910390f35b34801561017157600080fd5b506101b060048036038101908080\
3573ffffffffffffffffffffffffffffffffffffffff169060200190929190803590602001909291\
9050505061067e565b604051808215151515815260200191505060405180910390f35b3480156101\
d657600080fd5b506101df610770565b6040518082815260200191505060405180910390f35b3480\
1561020157600080fd5b50610260600480360381019080803573ffffffffffffffffffffffffffff\
ffffffffffff169060200190929190803573ffffffffffffffffffffffffffffffffffffffff1690\
60200190929190803590602001909291905050506107bb565b604051808215151515815260200191\
505060405180910390f35b34801561028657600080fd5b5061028f610a4b565b604051808260ff16\
60ff16815260200191505060405180910390f35b3480156102b757600080fd5b506102c0610a5e56\
5b6040518082815260200191505060405180910390f35b3480156102e257600080fd5b5061031760\
0480360381019080803573ffffffffffffffffffffffffffffffffffffffff169060200190929190\
505050610a64565b6040518082815260200191505060405180910390f35b34801561033957600080\
fd5b506103626004803603810190808035906020019092919080359060200190929190505050610a\
ad565b6040518082815260200191505060405180910390f35b34801561038457600080fd5b506103\
c3600480360381019080803573ffffffffffffffffffffffffffffffffffffffff16906020019092\
919080359060200190929190505050610ac9565b6040518082151515158152602001915050604051\
80910390f35b3480156103e957600080fd5b50610412600480360381019080803590602001909291\
9080359060200190929190505050610c52565b6040518082815260200191505060405180910390f3\
5b34801561043457600080fd5b506104b9600480360381019080803573ffffffffffffffffffffff\
ffffffffffffffffff16906020019092919080359060200190929190803590602001908201803590\
602001908080601f0160208091040260200160405190810160405280939291908181526020018383\
808284378201915050505050509192919290505050610c76565b6040518082151515158152602001\
91505060405180910390f35b3480156104df57600080fd5b50610508600480360381019080803590\
6020019092919080359060200190929190505050610ec5565b604051808281526020019150506040\
5180910390f35b34801561052a57600080fd5b5061057f600480360381019080803573ffffffffff\
ffffffffffffffffffffffffffffff169060200190929190803573ffffffffffffffffffffffffff\
ffffffffffffff169060200190929190505050610ef6565b60405180828152602001915050604051\
80910390f35b3480156105a157600080fd5b506105ca600480360381019080803590602001909291\
9080359060200190929190505050610f7d565b6040518082815260200191505060405180910390f3\
5b60008054600181600116156101000203166002900480601f016020809104026020016040519081\
01604052809291908181526020018280546001816001161561010002031660029004801561067657\
80601f1061064b57610100808354040283529160200191610676565b820191906000526020600020\
905b81548152906001019060200180831161065957829003601f168201915b505050505081565b60\
0081600460003373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffff\
ffffffffffffffffffff16815260200190815260200160002060008573ffffffffffffffffffffff\
ffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152\
602001600020819055508273ffffffffffffffffffffffffffffffffffffffff163373ffffffffff\
ffffffffffffffffffffffffffffff167f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b229\
1e5b200ac8c7c3b925846040518082815260200191505060405180910390a3600190509291505056\
5b6000600360008073ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffff\
ffffffffffffffffffffff1681526020019081526020016000205460025403905090565b60006108\
06600360008673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffff\
ffffffffffffffffff1681526020019081526020016000205483610aad565b600360008673ffffff\
ffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16\
8152602001908152602001600020819055506108cf600460008673ffffffffffffffffffffffffff\
ffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020\
0160002060003373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffff\
ffffffffffffffffffff1681526020019081526020016000205483610aad565b600460008673ffff\
ffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff\
16815260200190815260200160002060003373ffffffffffffffffffffffffffffffffffffffff16\
73ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002081905550\
610998600360008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffff\
ffffffffffffffffffffff1681526020019081526020016000205483610f7d565b600360008573ff\
ffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffff\
ff168152602001908152602001600020819055508273ffffffffffffffffffffffffffffffffffff\
ffff168473ffffffffffffffffffffffffffffffffffffffff167fddf252ad1be2c89b69c2b068fc\
378daa952ba7f163c4a11628f55a4df523b3ef846040518082815260200191505060405180910390\
a3600190509392505050565b600160009054906101000a900460ff1681565b60025481565b600060\
0360008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffff\
ffffffffffffff168152602001908152602001600020549050919050565b6000828211151515610a\
be57600080fd5b818303905092915050565b6000610b14600360003373ffffffffffffffffffffff\
ffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152\
6020016000205483610aad565b600360003373ffffffffffffffffffffffffffffffffffffffff16\
73ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002081905550\
610ba0600360008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffff\
ffffffffffffffffffffff1681526020019081526020016000205483610f7d565b600360008573ff\
ffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffff\
ff168152602001908152602001600020819055508273ffffffffffffffffffffffffffffffffffff\
ffff163373ffffffffffffffffffffffffffffffffffffffff167fddf252ad1be2c89b69c2b068fc\
378daa952ba7f163c4a11628f55a4df523b3ef846040518082815260200191505060405180910390\
a36001905092915050565b60008082111515610c6257600080fd5b8183811515610c6d57fe5b0490\
5092915050565b600082600460003373ffffffffffffffffffffffffffffffffffffffff1673ffff\
ffffffffffffffffffffffffffffffffffff16815260200190815260200160002060008673ffffff\
ffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16\
8152602001908152602001600020819055508373ffffffffffffffffffffffffffffffffffffffff\
163373ffffffffffffffffffffffffffffffffffffffff167f8c5be1e5ebec7d5bd14f71427d1e84\
f3dd0314c0f7b2291e5b200ac8c7c3b925856040518082815260200191505060405180910390a383\
73ffffffffffffffffffffffffffffffffffffffff16638f4ffcb1338530866040518563ffffffff\
167c0100000000000000000000000000000000000000000000000000000000028152600401808573\
ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffff\
ffff1681526020018481526020018373ffffffffffffffffffffffffffffffffffffffff1673ffff\
ffffffffffffffffffffffffffffffffffff16815260200180602001828103825283818151815260\
200191508051906020019080838360005b83811015610e5357808201518184015260208101905061\
0e38565b50505050905090810190601f168015610e805780820380516001836020036101000a0319\
16815260200191505b5095505050505050600060405180830381600087803b158015610ea2576000\
80fd5b505af1158015610eb6573d6000803e3d6000fd5b50505050600190509392505050565b6000\
81830290506000831480610ee55750818382811515610ee257fe5b04145b1515610ef057600080fd\
5b92915050565b6000600460008473ffffffffffffffffffffffffffffffffffffffff1673ffffff\
ffffffffffffffffffffffffffffffffff16815260200190815260200160002060008373ffffffff\
ffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681\
5260200190815260200160002054905092915050565b60008183019050828110151515610f935760\
0080fd5b929150505600a165627a7a723058201cf23aed1bd6b0855bf9debb24de21f8a443422637\
9c5614c9dcc004a62d902e0029";
