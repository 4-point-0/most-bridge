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
pub const MINTER_TOKEN_KEY: &str = "minter_token_local_key";

// This constant is our approximation of the expected header size.
// The HTTP standard doesn't define any limit, and many implementations limit
// the headers size to 8 KiB. We chose a lower limit because headers observed on most providers
// fit in the constant defined below, and if there is spike, then the payload size adjustment
// should take care of that.
const HEADER_SIZE_LIMIT: u64 = 2 * 1024;

// This constant comes from the IC specification:
// > If provided, the value must not exceed 2MB
const HTTP_MAX_SIZE: u64 = 2_000_000;

pub const MAX_PAYLOAD_SIZE: u64 = HTTP_MAX_SIZE - HEADER_SIZE_LIMIT;
