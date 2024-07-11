# MOST BRIDGE BACKEND

# Setting up ledger and minter locally

reference: https://internetcomputer.org/docs/current/developer-docs/defi/icp-tokens/ledger-local-setup

# Step 1: Download ledger canister locally in your repo:

```bash
curl -o download_latest_icp_ledger.sh "https://raw.githubusercontent.com/dfinity/ic/00a4ab409e6236d4082cee4a47544a2d87b7190d/rs/rosetta-api/scripts/download_latest_icp_ledger.sh"
chmod +x download_latest_icp_ledger.sh
./download_latest_icp_ledger.sh
```

# Step 2: Configure dfx.json:

```
{
  "canisters": {
    "icp_ledger_canister": {
      "type": "custom",
      "candid": icrc1_ledger.did,
      "wasm" : icrc1_ledger.wasm.gz,
      "specified_id": "mxzaz-hqaaa-aaaar-qaada-cai"
    }
  },
  "defaults": {
    "build": {
      "args": "",
      "packtool": ""
    }
  },
  "output_env_file": ".env",
  "version": 1
}
```

# Step 3: Start local replica:

```bash
dfx start --background --clean
```

# Step 4: Create a new identity that will work as a minting account

```bash
dfx identity new minter --storage-mode plaintext
dfx identity use minter
export MINTER=$(dfx identity get-principal)
```

# Step 5: Swithc back to your default identity

```bash
dfx identity use default
export DEFAULT=$(dfx identity get-principal)
```

# Step 6: Archive cycle

```bash
export CYCLE_FOR_ARCHIVE_CREATION=10000000000000
```

# Step 7: Deploy icrc1_ledger_canister:

```bash
dfx deploy ck_sui_ledger_canister --argument "(variant { Init =
record {
     token_symbol = \"CKSUI\";
     token_name = \"CKSUI\";
     decimals = opt 9;
     minting_account = record { owner = principal \"${MINTER}\" };
     transfer_fee = 10_000;
     metadata = vec {};
     initial_balances = vec { record { record { owner = principal \"${DEFAULT}\"; }; 100_000_000_000_000_000; }; };
     archive_options = record {
         num_blocks_to_archive = 1000;
         trigger_threshold = 2000;
         controller_id = principal \"${DEFAULT}\";
         cycles_for_archive_creation = opt ${CYCLE_FOR_ARCHIVE_CREATION};
     };
 }
})"
```

# Step 8: Deploy ircr1_ledger_indexer:

```bash
dfx deploy icrc1_index_canister --argument '(opt variant{Init = record { ledger_id = principal "mxzaz-hqaaa-aaaar-qaada-cai" }})'
```

# Step 9: Deploy minter:

```bash
dfx deploy minter --argument "( record {
  ledger_canister_id = \"${LEDGER_CANISTER_ID}\";
  local_mgmt_principal_id = \"${LOCAL_MGMT_PRINCIPAL_ID}\";
  sui_package_id = \"${SUI_PACKAGE_ID}\";
  sui_module_id = \"${SUI_MODULE_ID}\";
  sufinity_api_url = \"${SUFINITY_API_URL}\";
  tx_digest_url = \"${TX_DIGEST__URL}\";
  is_local = \"${IS_LOCAL}\";
  minter_address_id = \"${MINTER_ADDRESS_ID}\";
  })"
```

# Step 10: Transfer funds to canister

```bash
dfx canister call icrc1_ledger_canister icrc1_transfer "(record {
  to = record {
    owner = principal \"$(dfx canister id minter)\";
  };
  amount = 50_000_000_000_000_000;
})"
```

# Withdraw functionality

# 1 Step: Approve transfer

```bash
dfx canister call --identity ${PRINCIPAL_NAME} ck_sui_ledger_canister icrc2_approve "(
  record {
    spender= record {
      owner = principal \"$(dfx canister id minter)\";
    };
    amount = ${AMOUNT_IN_NAT}: nat;
  }
)"
```

- PRINCIPAL_NAME: for example default
- AMOUNT_IN_NAT: 1_000_000_000;

# 2 Step

```bash
 dfx canister call minter withdraw "(record {
  amount = \"${AMOUNT}\";
  recipient = \"${RECIPIENT_SUI _ADDRESS}\"
})"
```

- AMOUNT - string format 100000000 - it's like 0.1 SUI
- RECIPIENT_SUI \_ADDRESS - sui compatible address string
