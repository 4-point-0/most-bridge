export const minterFactory = ({ IDL }) => {
  const InitArgs = IDL.Record({
    sui_package_id: IDL.Text,
    sufinity_api_url: IDL.Text,
    tx_digest_url: IDL.Text,
    minter_address_id: IDL.Text,
    is_local: IDL.Text,
    sui_module_id: IDL.Text,
    ledger_canister_id: IDL.Text,
    local_mgmt_principal_id: IDL.Text,
  });
  const Account = IDL.Record({
    owner: IDL.Principal,
    subaccount: IDL.Opt(IDL.Vec(IDL.Nat8)),
  });
  const TransferArgsWithdraw = IDL.Record({
    recipient: IDL.Text,
    to_account: Account,
    amount: IDL.Text,
  });
  return IDL.Service({
    get_finalized_transactions: IDL.Func([], [IDL.Vec(IDL.Text)], []),
    get_minted_transactions: IDL.Func([], [IDL.Vec(IDL.Text)], []),
    public_key: IDL.Func(
      [],
      [
        IDL.Variant({
          Ok: IDL.Record({ public_key: IDL.Text }),
          Err: IDL.Text,
        }),
      ],
      []
    ),
    withdraw: IDL.Func(
      [TransferArgsWithdraw],
      [
        IDL.Variant({
          Ok: IDL.Record({ tx_digest: IDL.Text }),
          Err: IDL.Text,
        }),
      ],
      []
    ),
  });
};
export const init = ({ IDL }) => {
  const InitArgs = IDL.Record({
    sui_package_id: IDL.Text,
    sufinity_api_url: IDL.Text,
    tx_digest_url: IDL.Text,
    minter_address_id: IDL.Text,
    is_local: IDL.Text,
    sui_module_id: IDL.Text,
    ledger_canister_id: IDL.Text,
    local_mgmt_principal_id: IDL.Text,
  });
  return [InitArgs];
};
