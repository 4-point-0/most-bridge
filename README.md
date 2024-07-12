# MOST BRIDGE

This project consists of 4 repos:

- `most-bridge-backend` - canister for minting and withdrawing ckSUI tokens
- `most-bridge-helper` - generating sui address from base64 public key
- `most-ck-sui-helper` - sui smart contract for depositing SUI tokens to minter
- `most-bridge-api` - check here https://github.com/4-point-0/most-bridge-api

## Setup

- follow https://internetcomputer.org/docs/current/developer-docs/getting-started/install/ instructions to install necesarry tools
- for local testing deploy `most-ck-sui-helper` to local, devnet or testnet
- update constants `SUI_PACKAGE_ID` and `SUI_MODULE_ID` deploy canister from `most-bridge-backend` locally, get public key and convert it to SUI_ADDRESS using `most-bridge-helper`
- set minter address with `setMinterAddress` method on `ckSuiHelper.move` smart contract

## Bridge functionalities

### 1. Deposit

Use SUI cli script:

```bash
sui client call --package <CONTRACT_ID> --module ckSuiHelper --function deposit --args <AMOUNT> <COIN_ID> <PRINCIPAL_ADDRESS_STRING> <MINTER_ID>
```

### 2. Withdrawal

Export identity in cli:

```bash
dfx identity use default
export DEFAULT=$(dfx identity get-principal)
```

Approve amount for withdrawal:

```bash
dfx canister call --identity default ck_sui_ledger_canister  icrc2_approve "(record {spender= record {owner = principal \"${CANISTER_MINTER_ID}\";};amount = ${AMOUNT}: nat;})" --network ic
```

Withdraw

```bash
dfx canister call minter withdraw "(record {
  amount = \"${AMOUNT}\";
  recipient = \"${RECIPIENT_SUI _ADDRESS}\"
})"
```
