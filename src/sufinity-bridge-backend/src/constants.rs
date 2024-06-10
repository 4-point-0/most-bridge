use std::time::Duration;

pub const QUERY_SUI_EVENTS_INTERVAL: Duration = Duration::from_secs(60);
pub const LEDGER_CANISTER_ID: &str = "mxzaz-hqaaa-aaaar-qaada-cai";
pub const SUI_RPC_URL: &str = "https://fullnode.testnet.sui.io:443";
pub const PROCESSED_TX_DIGEST_KEY: &str = "txDigest";
// sui avg 30 DAYS TPS is 193 we are setting 193*60=11580 as we check how much events are there in 1 minute
pub const SUIX_QUERY_EVENTS: &str = "{\"jsonrpc\": \"2.0\",\"id\": 1,\"method\": \"suix_queryEvents\",\"params\":[{\"MoveModule\":{\"package\":\"0x44720817255b799b5c23d722568e850d07db51bf52b9c0425b3b16a1fe5f21a0\",\"module\": \"ckSuiHelper\"}},null,18000,false]}";
pub const SUFINITY_API_URL: &str = "https://local.sufinity:8080";
