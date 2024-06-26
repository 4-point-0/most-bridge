use serde_derive::Deserialize;
use serde_derive::Serialize;
use serde_json::Value;

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
pub struct ReferenceGasPriceRequest {
    pub jsonrpc: String,
    pub id: i64,
    pub method: String,
    pub params: Vec<Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceGasPriceResponse {
    pub jsonrpc: String,
    pub result: String,
    pub id: i64,
}
#[derive(Deserialize, Clone)]
pub struct TxDigestRequest {
    pub public_key: String,
    pub recipient: String,
    pub amount: String,
}
