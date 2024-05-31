# ICP denominations

- 1 ICP is equal to 10^8 e8

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

# Step 6: Deploy icrc1_ledger_canister:

```bash
dfx deploy icrc1_ledger_canister --argument "(variant { Init =
record {
     token_symbol = \"CKSUI\";
     token_name = \"L-CKSUI\";
     decimals = opt 9;
     minting_account = record { owner = principal \"${MINTER}\" };
     transfer_fee = 10_000;
     metadata = vec {};
     initial_balances = vec { record { record { owner = principal \"${DEFAULT}\"; }; 10_000_000_000; }; };
     archive_options = record {
         num_blocks_to_archive = 1000;
         trigger_threshold = 2000;
         controller_id = principal \"${MINTER}\";
     };
 }
})"
```

# Step 7: Deploy ircr1_ledger_indexer:

```bash
dfx deploy icrc1_index_canister --argument '(opt variant{Init = record { ledger_id = principal "mxzaz-hqaaa-aaaar-qaada-cai" }})'
```

# Step 8: Deploy minter:

```bash
dfx deploy
```

# Step 9: Transfer funds to canister

```bash
dfx canister call icrc1_ledger_canister icrc1_transfer "(record {
  to = record {
    owner = principal \"$(dfx canister id minter)\";
  };
  amount = 1_000_000_000;
})"
```
