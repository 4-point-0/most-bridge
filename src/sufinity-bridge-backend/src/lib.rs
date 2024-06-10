use common::{
    Context, ECDSAPublicKey, ECDSAPublicKeyReply, EcdsaKeyIds, SignWithECDSA, SignWithECDSAReply,
    SignatureReply, TxDigestResponse,
};
use constants::{
    LEDGER_CANISTER_ID, PROCESSED_TX_DIGEST_KEY, QUERY_SUI_EVENTS_INTERVAL, SUFINITY_API_URL,
    SUIX_QUERY_EVENTS, SUI_RPC_URL,
};
use event::{Event, KeyName, KeyValue, Memory};
use ic_canister_log::log;
use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod, HttpResponse, TransformArgs,
    TransformContext,
};
use ic_cdk::{query, update};
use icrc_ledger_types::icrc1::transfer::NumTokens;
use serde_json::{self};
use std::str::FromStr;
mod common;
mod constants;
mod event;
mod logs;
use crate::logs::INFO;
use base64::{self, engine::general_purpose::STANDARD, Engine};
use candid::{candid_method, Principal};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap};
use std::cell::RefCell;
mod receipt;

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

async fn mint() {
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
        suix_query_events = format!("{{\"jsonrpc\": \"2.0\",\"id\": 1,\"method\": \"suix_queryEvents\",\"params\":[{{\"MoveModule\":{{\"package\":\"0x44720817255b799b5c23d722568e850d07db51bf52b9c0425b3b16a1fe5f21a0\",\"module\": \"ckSuiHelper\"}}}},{{\"txDigest\":\"{}\", \"eventSeq\": \"0\"}},2,false]}}", tx_digest_value);
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
        transform: Some(TransformContext::from_name(
            "cleanup_response".to_owned(),
            serde_json::to_vec(&context).unwrap(),
        )),
    };

    match http_request(request, 25_000_000_000).await {
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

#[query]
#[candid_method(query)]
fn cleanup_response(raw: TransformArgs) -> HttpResponse {
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

    if res.status == 200u16 {
        res.body = raw.response.body.clone();
    } else {
        ic_cdk::api::print(format!("Received an error from coinbase: err = {:?}", raw));
    }
    res
}

#[update]
async fn withdraw(msg: String) -> Result<SignatureReply, String> {
    let context = Context {
        bucket_start_time_index: 0,
        closing_price_index: 4,
    };

    let mut public_key: String = String::new();
    let mut signature: String = "".to_string();

    let request = ECDSAPublicKey {
        canister_id: None,
        derivation_path: vec![],
        key_id: EcdsaKeyIds::TestKeyLocalDevelopment.to_key_id(),
    };

    let (res_public_key,): (ECDSAPublicKeyReply,) =
        ic_cdk::call(mgmt_canister_id(), "ecdsa_public_key", (request,))
            .await
            .map_err(|e| format!("ecdsa_public_key failed {}", e.1))?;

    let request = CanisterHttpRequestArgument {
        url: "https://local.sufinity:8080/tx-digest".to_string(),
        max_response_bytes: None,
        method: HttpMethod::GET,
        headers: vec![HttpHeader {
            name: "Host".to_string(),
            value: SUFINITY_API_URL.to_string(),
        }],
        body: None,
        transform: Some(TransformContext::from_name(
            "cleanup_response".to_owned(),
            serde_json::to_vec(&context).unwrap(),
        )),
    };

    match http_request(request, 25_000_000_000).await {
        Ok((response,)) => {
            let result = serde_json::from_slice::<TxDigestResponse>(&response.body)
                .map_err(|e| format!("Error: {}", e.to_string()))
                .unwrap();

            let digest_decoded = Engine::decode(&STANDARD, result.digest)
                .map_err(|e| format!("Error: {}", e.to_string()))
                .unwrap();

            let digest: [u8; 32] = digest_decoded.try_into().unwrap();

            let request = SignWithECDSA {
                message_hash: sha256(digest).to_vec(),
                derivation_path: vec![],
                key_id: EcdsaKeyIds::TestKeyLocalDevelopment.to_key_id(),
            };

            let (response,): (SignWithECDSAReply,) = ic_cdk::api::call::call_with_payment(
                mgmt_canister_id(),
                "sign_with_ecdsa",
                (request,),
                25_000_000_000,
            )
            .await
            .map_err(|e| format!("sign_with_ecdsa failed {}", e.1))?;
            public_key = Engine::encode(&STANDARD, &res_public_key.public_key.clone());

            let flag: u8 = 0x1;
            let mut signature_bytes: Vec<u8> = Vec::new();
            signature_bytes.extend_from_slice(&[flag]);
            signature_bytes.extend_from_slice(&response.signature.as_ref());
            signature_bytes.extend_from_slice(&res_public_key.public_key.as_ref());

            signature = Engine::encode(&STANDARD, &signature_bytes[..]);
        }
        Err((r, m)) => {
            log!(
                INFO,
                "The http_request resulted into error. RejectionCode: {r:?}, Error: {m}"
            );
        }
    };
    Ok(SignatureReply {
        public_key,
        signature,
    })
}

fn sha256(input: [u8; 32]) -> [u8; 32] {
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(input);
    hasher.finalize().into()
}

fn mgmt_canister_id() -> Principal {
    Principal::from_str(&"aaaaa-aa").unwrap()
}

fn insert_event(key: String, value: Event) -> Option<Event> {
    EVENTS.with(|p| p.borrow_mut().insert(KeyName(key), value))
}

fn get(key: String) -> Option<String> {
    MAP.with(|p| p.borrow().get(&KeyName(key)).map(|v| v.0))
}

fn insert(key: String, value: String) -> Option<String> {
    MAP.with(|p| p.borrow_mut().insert(KeyName(key), KeyValue(value)))
        .map(|v| v.0)
}

// #[ic_cdk::query]
// async fn total_events() -> u64 {
//     EVENTS.with(|events| events.borrow().len())
// }

// #[ic_cdk::query]
// async fn get_one() -> String {
//     EVENTS.with(|events| {
//         events.borrow_mut().iter().for_each(|(k, v)| {
//             log!(INFO, "Key: {}, Value: {:?}", k.0, v.tx_digest);
//         })
//     });
//     return "test".to_string();
// }

// #[ic_cdk::query]
// async fn get_events() -> Vec<(String, String)> {
//     EVENTS.with(|t| {
//         t.borrow()
//             .iter()
//             .map(|(k, v)| (k.clone().0.to_string(), v.tx_digest))
//             .collect()
//     })
// }

// fn from_bytes(bytes: &[u8]) -> [u8; 98] {
//     const LENGTH: usize = 98;
//     let mut sig_bytes: [u8; 98] = [0; LENGTH];
//     sig_bytes.copy_from_slice(bytes);
//     sig_bytes
// }

// async fn get_public_key() -> Result<Vec<u8>, String> {
//     let context = Context {
//         bucket_start_time_index: 0,
//         closing_price_index: 4,
//     };
//     let public_key_req = CanisterHttpRequestArgument {
//         url: "https://local.sufinity:8080/public-key".to_string(),
//         max_response_bytes: None,
//         method: HttpMethod::GET,
//         headers: vec![HttpHeader {
//             name: "Host".to_string(),
//             value: format!("https://local.sufinity:8080"),
//         }],
//         body: None,
//         transform: Some(TransformContext::from_name(
//             "cleanup_response".to_owned(),
//             serde_json::to_vec(&context).unwrap(),
//         )),
//     };

//     let pub_key = match http_request(public_key_req, 25_000_000_000).await {
//         Ok((response,)) => {
//             let public_key = &response.body;
//             Ok(public_key.clone())
//         }
//         Err((r, m)) => {
//             let message =
//                 format!("The http_request resulted into error. RejectionCode: {r:?}, Error: {m}");

//             //Return the error as a string and end the method
//             Err(message)
//         }
//     };
//     return pub_key;
// }
