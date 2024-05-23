use ic_canister_log::log;
use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod, HttpResponse, TransformArgs,
    TransformContext,
};
use ic_cdk_macros::{self, query};
use serde::Serialize;
use serde_json::{self};
use std::borrow::Cow;
use std::str::FromStr;
mod logs;
use crate::logs::INFO;
use candid::{CandidType, Decode, Deserialize, Encode};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{storable::Bound, DefaultMemoryImpl, StableBTreeMap, Storable};
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc1::transfer::NumTokens;
use std::cell::RefCell;
use std::time::Duration;

mod receipt;

#[derive(CandidType, Deserialize, Serialize)]
pub struct TransferArgs {
    amount: NumTokens,
    to_account: Account,
}

#[derive(Serialize, Deserialize)]
struct Context {
    bucket_start_time_index: usize,
    closing_price_index: usize,
}

pub const QUERY_SUI_EVENTS_INTERVAL: Duration = Duration::from_secs(60);
const LEDGER_CANISTER_ID: &str = "mxzaz-hqaaa-aaaar-qaada-cai";
const SUI_RPC_URL: &str = "https://fullnode.testnet.sui.io:443";
const PROCESSED_TX_DIGEST_KEY: &str = "txDigest";
// sui avg 30 DAYS TPS is 193 we are setting 193*60=11580 as we check how much events are there in 1 minute
const SUIX_QUERY_EVENTS: &str = "{\"jsonrpc\": \"2.0\",\"id\": 1,\"method\": \"suix_queryEvents\",\"params\":[{\"MoveModule\":{\"package\":\"0x44720817255b799b5c23d722568e850d07db51bf52b9c0425b3b16a1fe5f21a0\",\"module\": \"ckSuiHelper\"}},null,18000,false]}";

#[derive(CandidType, PartialEq, Deserialize)]
struct Event {
    timestamp: u64,
    tx_digest: String,
    from: String,
    minter_address: String,
    principal_address: String,
    value: String,
}

type Memory = VirtualMemory<DefaultMemoryImpl>;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
struct KeyName(String);

impl Storable for KeyName {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        self.0.to_bytes()
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Self(String::from_bytes(bytes))
    }

    const BOUND: Bound = Bound::Unbounded;
}

struct KeyValue(String);

impl Storable for KeyValue {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        self.0.to_bytes()
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Self(String::from_bytes(bytes))
    }

    const BOUND: Bound = Bound::Unbounded;
}

impl Storable for Event {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));


    static MAP: RefCell<StableBTreeMap<KeyName, KeyValue, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
        )
    );

    static EVENTS: RefCell<StableBTreeMap<KeyName, Event, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
        )
    );

}

fn setup_timers() {
    ic_cdk_timers::set_timer_interval(QUERY_SUI_EVENTS_INTERVAL, || ic_cdk::spawn(self::mint()));
    log!(INFO, "Timer set");
}

#[ic_cdk_macros::init]
fn init() {
    setup_timers();
}

#[ic_cdk::query]
async fn total_events() -> u64 {
    EVENTS.with(|events| events.borrow().len())
}

#[ic_cdk::query]
async fn get_one() -> String {
    EVENTS.with(|events| {
        events.borrow_mut().iter().for_each(|(k, v)| {
            log!(INFO, "Key: {}, Value: {:?}", k.0, v.tx_digest);
        })
    });
    return "test".to_string();
}

#[ic_cdk::query]
async fn get_events() -> Vec<(String, String)> {
    EVENTS.with(|t| {
        t.borrow()
            .iter()
            .map(|(k, v)| (k.clone().0.to_string(), v.tx_digest))
            .collect()
    })
}

async fn mint() {
    use candid::Principal;
    use icrc_ledger_client::{CdkRuntime, ICRC1Client};
    use icrc_ledger_types::icrc1::account::Account;
    use icrc_ledger_types::icrc1::transfer::TransferArg;

    let context = Context {
        bucket_start_time_index: 0,
        closing_price_index: 4,
    };

    let tx_digest_cursor = self::get(PROCESSED_TX_DIGEST_KEY.to_string());
    let mut suix_query_events = SUIX_QUERY_EVENTS.to_string();
    let mut tx_digest_value = "";

    if tx_digest_cursor != None {
        tx_digest_value = tx_digest_cursor.as_ref().unwrap();
        suix_query_events = format!(
            "{{\"jsonrpc\": \"2.0\",\"id\": 1,\"method\": \"suix_queryEvents\",\"params\":[{{\"MoveModule\":{{\"package\":\"0x44720817255b799b5c23d722568e850d07db51bf52b9c0425b3b16a1fe5f21a0\",\"module\": \"ckSuiHelper\"}}}},{{\"txDigest\":\"{}\", \"eventSeq\": \"0\"}},2,false]}}",
            tx_digest_value
        );
    }

    let request = CanisterHttpRequestArgument {
        url: SUI_RPC_URL.to_string(),
        max_response_bytes: None,
        method: HttpMethod::POST,
        headers: vec![HttpHeader {
            name: "Content-Type".to_string(),
            value: "application/json".to_string(),
        }],
        body: Some(suix_query_events.to_string().into_bytes()),
        transform: Some(TransformContext::new(
            transform,
            serde_json::to_vec(&context).unwrap(),
        )),
    };

    match http_request(request).await {
        Ok((response,)) => {
            let trasnaction = serde_json::from_slice::<receipt::Root>(&response.body)
                .map_err(|e| format!("Error: {}", e.to_string()));

            let events = &trasnaction.clone().unwrap().result.data;

            for event in events {
                if tx_digest_value == event.id.tx_digest {
                    break;
                }

                let parsed_json = &event.parsed_json;

                let principal: Principal =
                    Principal::from_str(&parsed_json.principal_address).unwrap();
                let amount: NumTokens = NumTokens::from_str(&parsed_json.value).unwrap();

                let ledger_canister_id: Principal =
                    Principal::from_text(LEDGER_CANISTER_ID).unwrap();

                let client = ICRC1Client {
                    runtime: CdkRuntime,
                    ledger_canister_id,
                };

                let canister_backend = Account {
                    owner: Principal::from_text(ic_cdk::id().to_string()).unwrap(),
                    subaccount: None,
                };

                let balance = client.balance_of(canister_backend).await.unwrap();

                if balance < amount {
                    log!(INFO, "Not enough balance ({balance})");
                    continue;
                }

                let to: Account = Account {
                    owner: principal,
                    subaccount: None,
                };
                let block_index = match client
                    .transfer(TransferArg {
                        from_subaccount: None,
                        to,
                        fee: None,
                        created_at_time: None,
                        memo: None,
                        amount,
                    })
                    .await
                {
                    Ok(Ok(block_index)) => {
                        self::insert(
                            PROCESSED_TX_DIGEST_KEY.to_string(),
                            trasnaction.clone().unwrap().result.next_cursor.tx_digest,
                        );

                        self::insert_event(
                            event.id.tx_digest.to_string(),
                            Event {
                                timestamp: ic_cdk::api::time(),
                                tx_digest: event.id.tx_digest.to_string(),
                                from: parsed_json.from.to_string(),
                                minter_address: parsed_json.minter_address.to_string(),
                                principal_address: parsed_json.principal_address.to_string(),
                                value: parsed_json.value.to_string(),
                            },
                        );
                        block_index.to_string()
                    }
                    Ok(Err(err)) => err.to_string(),
                    Err(err) => {
                        log!(
                                INFO,
                                "Failed to send a message to the ledger ({ledger_canister_id}): {err:?}"
                            );
                        "error".to_string()
                    }
                };

                log!(INFO, "Minted tokens on ({block_index})");
            }
        }
        Err((r, m)) => {
            log!(
                INFO,
                "The http_request resulted into error. RejectionCode: {r:?}, Error: {m}"
            );
        }
    }
}

// Strips all data that is not needed from the original response.
#[query]
fn transform(raw: TransformArgs) -> HttpResponse {
    let headers = vec![
        HttpHeader {
            name: "Content-Security-Policy".to_string(),
            value: "default-src 'self'".to_string(),
        },
        HttpHeader {
            name: "Referrer-Policy".to_string(),
            value: "strict-origin".to_string(),
        },
        HttpHeader {
            name: "Permissions-Policy".to_string(),
            value: "geolocation=(self)".to_string(),
        },
        HttpHeader {
            name: "Strict-Transport-Security".to_string(),
            value: "max-age=63072000".to_string(),
        },
        HttpHeader {
            name: "X-Frame-Options".to_string(),
            value: "DENY".to_string(),
        },
        HttpHeader {
            name: "X-Content-Type-Options".to_string(),
            value: "nosniff".to_string(),
        },
    ];

    let mut res = HttpResponse {
        status: raw.response.status.clone(),
        body: raw.response.body.clone(),
        headers,
        ..Default::default()
    };

    if res.status == 200 {
        res.body = raw.response.body.clone();
    } else {
        ic_cdk::api::print(format!("Received an error from coinbase: err = {:?}", raw));
    }
    res
}

fn insert_event(key: String, value: Event) -> Option<Event> {
    EVENTS.with(|p| p.borrow_mut().insert(KeyName(key), value))
}

// Gets the value of the key if it exists.
fn get(key: String) -> Option<String> {
    MAP.with(|p| p.borrow().get(&KeyName(key)).map(|v| v.0))
}

// Inserts an entry into the map and returns the previous value of the key if it exists.
fn insert(key: String, value: String) -> Option<String> {
    MAP.with(|p| p.borrow_mut().insert(KeyName(key), KeyValue(value)))
        .map(|v| v.0)
}
