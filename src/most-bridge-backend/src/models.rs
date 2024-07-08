use candid::CandidType;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use std::fmt;

use crate::constants::MAX_PAYLOAD_SIZE;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaySuiRequest {
    pub jsonrpc: String,
    pub id: i64,
    pub method: String,
    pub params: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Receipt {
    pub jsonrpc: String,
    pub result: ReceiptResult,
    pub id: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceiptResult {
    pub data: Vec<ReceiptResultData>,
    pub next_cursor: NextCursor,
    pub has_next_page: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceiptResultData {
    pub id: Id,
    pub package_id: String,
    pub transaction_module: String,
    pub sender: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub parsed_json: ParsedJson,
    pub bcs: String,
    pub timestamp_ms: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Id {
    pub tx_digest: String,
    pub event_seq: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedJson {
    pub from: String,
    #[serde(rename = "minter_address")]
    pub minter_address: String,
    #[serde(rename = "principal_address")]
    pub principal_address: String,
    pub value: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NextCursor {
    pub tx_digest: String,
    pub event_seq: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaySuiResponse {
    pub jsonrpc: String,
    pub result: PaySuiResponseResult,
    pub id: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaySuiResponseResult {
    pub tx_bytes: String,
    pub gas: Vec<Gas>,
    pub input_objects: Vec<InputObject>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Gas {
    pub object_id: String,
    pub version: i64,
    pub digest: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputObject {
    #[serde(rename = "ImmOrOwnedMoveObject")]
    pub imm_or_owned_move_object: ImmOrOwnedMoveObject,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImmOrOwnedMoveObject {
    pub object_id: String,
    pub version: i64,
    pub digest: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MergeCoinsRequest {
    pub primary_coin: String,
    pub secundary_coin: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TxDigestRequest {
    pub recipient: String,
    pub amount: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteTxBlock {
    pub jsonrpc: String,
    pub id: i64,
    pub method: String,
    pub params: (String, Vec<String>, Option<TxBlockParams>, Option<String>),
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TxBlockParams {
    pub show_input: bool,
    pub show_raw_input: bool,
    pub show_effects: bool,
    pub show_events: bool,
    pub show_object_changes: bool,
    pub show_balance_changes: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteTxBlockResponse {
    pub jsonrpc: String,
    pub result: ExecuteTxBlockResponseResult,
    pub id: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteTxBlockResponseResult {
    pub digest: String,
    pub confirmed_local_execution: bool,
}

#[derive(Serialize, Debug)]
pub struct PublicKeyResponse {
    pub public_key: Vec<u8>,
    pub public_key_bs64: String,
}

#[derive(CandidType, Serialize, Debug)]
pub struct PublicKeyBS64 {
    pub public_key: String,
}

#[derive(CandidType, Deserialize, Serialize)]
pub struct TransferWithdrawArgs {
    pub amount: String,
    pub recipient: String,
}

#[derive(CandidType, Deserialize, Serialize)]
pub struct InitArgs {
    pub ledger_canister_id: String,
    pub local_mgmt_principal_id: String,
    pub sui_package_id: String,
    pub sui_module_id: String,
    pub sufinity_api_url: String,
    pub tx_digest_url: String,
    pub is_local: String,
    pub minter_address_id: String,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResponseSizeEstimate(u64);

impl ResponseSizeEstimate {
    pub fn new(num_bytes: u64) -> Self {
        assert!(num_bytes > 0);
        assert!(num_bytes <= MAX_PAYLOAD_SIZE);
        Self(num_bytes)
    }

    /// Describes the expected (90th percentile) number of bytes in the HTTP response body.
    /// This number should be less than `MAX_PAYLOAD_SIZE`.
    pub fn get(self) -> u64 {
        self.0
    }

    /// Returns a higher estimate for the payload size.
    pub fn adjust(self) -> Self {
        Self(self.0.max(1024).saturating_mul(2).min(MAX_PAYLOAD_SIZE))
    }
}

impl fmt::Display for ResponseSizeEstimate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
