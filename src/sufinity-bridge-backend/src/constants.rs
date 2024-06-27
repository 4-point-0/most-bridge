use std::time::Duration;

pub const QUERY_SUI_EVENTS_INTERVAL: Duration = Duration::from_secs(60);
pub const SUI_TESTNET_RPC_URL: &str = "https://fullnode.testnet.sui.io:443";
pub const SUI_MAINNET_RPC_URL: &str = "https://fullnode.mainnet.sui.io:443";
pub const PROCESSED_TX_DIGEST_KEY: &str = "txDigest";
pub const LEDGER_CANISTER_ID_KEY: &str = "ledger_canister_id_key";
pub const LOCAL_MGMT_PRINCIPAL_ID_KEY: &str = "local_mgmt_principal_id_key";
pub const SUI_PACKAGE_ID_KEY: &str = "sui_package_id_key";
pub const SUI_MODULE_ID_KEY: &str = "sui_module_id_key";
pub const API_URL_KEY: &str = "api_url_key";
pub const TX_DIGEST_URL_KEY: &str = "tx_digest_url_key";
pub const IS_LOCAL_KEY: &str = "is_local_key";
