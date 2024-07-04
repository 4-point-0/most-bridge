# MOST BRIDGE

This project consists of 4 repos:

- `most-bridge-api` - api that provides data for querying SUI_RPC
- `most-bridge-backend` - canister for minting and withdrawing ckSUI tokens
- `most-bridge-helper` - generating sui address from base64 public key
- `most-ck-sui-helper` - sui smart contract for depositing SUI tokens to minter

# Setup

- follow https://internetcomputer.org/docs/current/developer-docs/getting-started/install/ instructions to install necesarry tools
- for local testing deploy `most-ck-sui-helper` to local, devnet or testnet
- update constants `SUI_PACKAGE_ID` and `SUI_MODULE_ID` deploy canister from `most-bridge-backend` locally, get public key and convert it to SUI_ADDRESS using `most-bridge-helper`
- set minter address with `setMinterAddress` method on `ckSuiHelper.move` smart contract

# Bridge functionalities

1.  Deposit funds to `deposit` function to `ckSuiHelper.move` smart contract
2.  Withdraw funds via `withdraw` function on `minter` canister
