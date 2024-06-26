## Running the CkSuiHelper Contract

1. Publish the contract

```bash
sui client publish --gas-budget 1000000000 --skip-dependency-verification
```

2. Collect your AdminCap & Minter from the transaction and change the minter address:

```bash
sui client call --package <CONTRACT_ID> --module ckSuiHelper --function setMinterAddress --args <ADMIN_CAP> <NEW_MINTER_ADDRESS> <MINTER_ID>
```

3. Use the deposit method:

```bash
sui client call --package <CONTRACT_ID> --module ckSuiHelper --function deposit --args <AMOUNT> <COIN_ID> <PRINCIPAL_ADDRESS_STRING> <MINTER_ID>
```
