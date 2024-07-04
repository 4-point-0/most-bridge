use common::{
    Context, ECDSAPublicKey, ECDSAPublicKeyReply, EcdsaKeyIds, SignWithECDSA, SignWithECDSAReply,
    TxDigestResponse, WithdrawResponse,
};
use constants::{
    LEDGER_CANISTER_ID_KEY, PROCESSED_TX_DIGEST_KEY, QUERY_SUI_EVENTS_INTERVAL,
    SUI_MAINNET_RPC_URL, SUI_MODULE_ID_KEY, SUI_PACKAGE_ID_KEY, SUI_TESTNET_RPC_URL,
};
use helper::{KeyName, KeyValue, Memory};
use ic_canister_log::log;
use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod, HttpResponse, TransformArgs,
    TransformContext,
};
use ic_cdk::{api, query, update};
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc1::transfer::NumTokens;
use icrc_ledger_types::icrc2::transfer_from::TransferFromArgs;
use models::{
    ExecuteTxBlockResponse, InitArgs, PublicKeyBS64, PublicKeyResponse, ResponseSizeEstimate,
    TransferWithdrawArgs, TxDigestRequest,
};
use serde_json::{self};
use std::str::FromStr;
mod common;
mod constants;
mod helper;
mod logs;
use crate::logs::INFO;
use base64::{self, engine::general_purpose::STANDARD, Engine};
use candid::{candid_method, Nat, Principal};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap};
use std::cell::RefCell;
pub mod models;
use crate::models::Receipt;
use icrc_ledger_types::icrc1::transfer::BlockIndex;
use icrc_ledger_types::icrc2::transfer_from::TransferFromError;

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));


    static MAP: RefCell<StableBTreeMap<KeyName, KeyValue, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
        )
    );

    static FINALIZED_TRANSACTIONS: RefCell<StableBTreeMap<KeyName, KeyValue, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))),
        )
    );

    static MINTED_TRANSACTIONS: RefCell<StableBTreeMap<KeyName, KeyValue, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2))),
        )
    );
}

fn setup_timers() {
    ic_cdk_timers::set_timer_interval(QUERY_SUI_EVENTS_INTERVAL, || ic_cdk::spawn(self::mint()));
}

#[ic_cdk_macros::init]
fn init(args: InitArgs) {
    setup_timers();
    let InitArgs {
        ledger_canister_id,
        local_mgmt_principal_id,
        sui_package_id,
        sui_module_id,
        sufinity_api_url,
        tx_digest_url,
        is_local,
        minter_address_id,
    } = args;

    if ledger_canister_id == ""
        || local_mgmt_principal_id == ""
        || sui_package_id == ""
        || sui_module_id == ""
        || sufinity_api_url == ""
        || tx_digest_url == ""
        || is_local == ""
        || minter_address_id == ""
    {
        log!(INFO, "Missing required arguments");
        return;
    }
    self::insert(
        constants::LEDGER_CANISTER_ID_KEY.to_string(),
        ledger_canister_id,
    );
    self::insert(
        constants::LOCAL_MGMT_PRINCIPAL_ID_KEY.to_string(),
        local_mgmt_principal_id,
    );
    self::insert(constants::SUI_PACKAGE_ID_KEY.to_string(), sui_package_id);
    self::insert(constants::SUI_MODULE_ID_KEY.to_string(), sui_module_id);
    self::insert(constants::API_URL_KEY.to_string(), sufinity_api_url);
    self::insert(constants::TX_DIGEST_URL_KEY.to_string(), tx_digest_url);
    self::insert(constants::IS_LOCAL_KEY.to_string(), is_local.to_string());
    self::insert(
        constants::MINTER_TOKEN_KEY.to_string(),
        minter_address_id.to_string(),
    );
}

#[update]
async fn public_key() -> Result<PublicKeyBS64, String> {
    return Ok(PublicKeyBS64 {
        public_key: get_public_key().await.unwrap().public_key_bs64,
    });
}

#[update]
async fn withdraw(args: TransferWithdrawArgs) -> Result<WithdrawResponse, String> {
    let token_minter = self::get(constants::MINTER_TOKEN_KEY.to_string()).unwrap();

    let transfer_from_args = TransferFromArgs {
        from: Account::from(ic_cdk::caller()),
        memo: None,
        amount: Nat::from_str(&args.amount.clone()).unwrap(),
        spender_subaccount: None,
        fee: None,
        to: Account {
            owner: Principal::from_text(token_minter).unwrap(),
            subaccount: None,
        },
        created_at_time: None,
    };

    let result = ic_cdk::call::<(TransferFromArgs,), (Result<BlockIndex, TransferFromError>,)>(
        Principal::from_text(self::get(LEDGER_CANISTER_ID_KEY.to_string()).unwrap()).unwrap(),
        "icrc2_transfer_from",
        (transfer_from_args,),
    )
    .await
    .map_err(|e| format!("failed to call ledger: {:?}", e))?
    .0
    .map_err(|e| format!("ledger transfer error {:?}", e));

    if result.is_err() {
        return Err(result.unwrap_err());
    }

    let block_index = result.unwrap().to_string();

    let public_key_response = get_public_key()
        .await
        .map_err(|e| format!("get_public_key failed {:?}", e));

    if public_key_response.is_err() {
        return Err(public_key_response.unwrap_err());
    }

    let public_key = public_key_response.unwrap().public_key;

    let request = get_withdraw_request(public_key.clone(), args.recipient, args.amount.clone());
    let cycles = get_req_cycles();
    match http_request(request, cycles).await {
        Ok((response,)) => {
            let result = serde_json::from_slice::<TxDigestResponse>(&response.body);

            if result.is_err() {
                return Err("Failed to get tx digest".to_string());
            }

            let TxDigestResponse { digest, tx_bytes } = result.unwrap();

            let signature = encode_signature(digest.clone(), public_key.clone()).await;
            let tx_digest = execute_tx_block_sui_rpc(signature.clone(), tx_bytes.clone())
                .await
                .map_err(|e| format!("execute_tx_block_sui_rpc error {}", e))?;
            let is_local = self::get(constants::IS_LOCAL_KEY.to_string()).unwrap();
            let tx: String = match is_local.as_str() {
                "true" => format!(
                    "https://suiscan.xyz/{:}/tx/{:}",
                    "testnet",
                    tx_digest.clone()
                ),
                _ => format!(
                    "https://suiscan.xyz/{:}/tx/{:}",
                    "mainnet",
                    tx_digest.clone()
                ),
            };

            self::insert_withdraw_tx(
                block_index.clone(),
                format!("{{\"block_index\": \"{:}\",\"date\":\"{:}\", \"amount\": \"{:}\",\"from\": \"{:}\", \"tx\": \"{:}\" }}"
                ,block_index.clone(), api::time().to_string(), args.amount, ic_cdk::caller().to_string(), tx
            ));

            return Ok(WithdrawResponse { tx_digest });
        }
        Err((r, m)) => {
            log!(
                INFO,
                "The http_request resulted into error. RejectionCode: {r:?}, Error: {m}"
            );
            return Err(m.to_string());
        }
    };
}

#[candid_method(query)]
#[query]
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
        ic_cdk::api::print(format!("Received error: err = {:?}", raw));
    }
    res
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
    let mut suix_query_events = format!("{{\"jsonrpc\": \"2.0\",\"id\": 1,\"method\": \"suix_queryEvents\",\"params\":[{{\"MoveModule\":{{\"package\":\"{}\",\"module\": \"{}\"}}}},null,18000,false]}}",self::get(SUI_PACKAGE_ID_KEY.to_string()).unwrap(), self::get(SUI_MODULE_ID_KEY.to_string()).unwrap());
    let mut tx_digest_value = "";

    if tx_digest_cursor != None {
        tx_digest_value = tx_digest_cursor.as_ref().unwrap();
        suix_query_events = format!("{{\"jsonrpc\": \"2.0\",\"id\": 1,\"method\": \"suix_queryEvents\",\"params\":[{{\"MoveModule\":{{\"package\":\"{}\",\"module\": \"{}\"}}}},{{\"txDigest\":\"{}\", \"eventSeq\": \"0\"}},2,false]}}",self::get(SUI_PACKAGE_ID_KEY.to_string()).unwrap(), self::get(SUI_MODULE_ID_KEY.to_string()).unwrap(), tx_digest_value);
    }

    let effective_size_estimate = get_effective_size_estimate();
    let cycles = get_req_cycles();
    let sui_rpc_url = get_sui_rpc_url();
    let request = CanisterHttpRequestArgument {
        url: sui_rpc_url,
        max_response_bytes: Some(effective_size_estimate),
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

    match http_request(request, cycles).await {
        Ok((response,)) => {
            let trasnaction = serde_json::from_slice::<Receipt>(&response.body)
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
                    Principal::from_text(self::get(LEDGER_CANISTER_ID_KEY.to_string()).unwrap())
                        .unwrap();

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

                        self::insert_mint_tx(
                            block_index.clone().to_string(),
                            format!("{{\"block_index\": \"{:}\",\"date\":\"{:}\", \"amount\": \"{:}\",\"from\": \"{:}\", \"to\": \"{:}\" }}"
                            ,block_index.clone(), api::time().to_string(), &parsed_json.value, ic_cdk::caller().to_string(), &parsed_json.principal_address
                        ));
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

async fn encode_signature(digest: String, public_key: Vec<u8>) -> String {
    let digest_decoded = Engine::decode(&STANDARD, digest)
        .map_err(|e| format!("Error: {}", e.to_string()))
        .unwrap();

    let digest: [u8; 32] = digest_decoded.try_into().unwrap();
    let signature = sign_with_ecdsa(digest).await;

    let flag: u8 = 0x1;
    let mut signature_bytes: Vec<u8> = Vec::new();
    signature_bytes.extend_from_slice(&[flag]);
    signature_bytes.extend_from_slice(&signature.as_ref());
    signature_bytes.extend_from_slice(&public_key.as_ref());

    let signature_encoded = Engine::encode(&STANDARD, &signature_bytes[..]);
    return signature_encoded;
}

fn get_withdraw_request(
    public_key: Vec<u8>,
    recipient: String,
    amount: String,
) -> CanisterHttpRequestArgument {
    let context = Context {
        bucket_start_time_index: 0,
        closing_price_index: 4,
    };
    let pub_key = Engine::encode(&STANDARD, &public_key);
    let model = TxDigestRequest {
        public_key: pub_key.clone(),
        recipient,
        amount,
    };
    let json_string: String = match serde_json::to_string(&model) {
        Ok(resp) => resp,
        Err(err) => panic!("Failed to serialize: {}", err.to_string()),
    };
    let json_utf8: Vec<u8> = json_string.into_bytes();
    let request_body: Option<Vec<u8>> = Some(json_utf8);
    let effective_size_estimate = get_effective_size_estimate();
    let request = CanisterHttpRequestArgument {
        url: self::get(constants::TX_DIGEST_URL_KEY.to_string()).unwrap(),
        max_response_bytes: Some(effective_size_estimate),
        method: HttpMethod::POST,
        headers: vec![
            HttpHeader {
                name: "Host".to_string(),
                value: self::get(constants::API_URL_KEY.to_string()).unwrap(),
            },
            HttpHeader {
                name: "Content-Type".to_string(),
                value: "application/json".to_string(),
            },
        ],
        body: request_body,
        transform: Some(TransformContext::from_name(
            "cleanup_response".to_owned(),
            serde_json::to_vec(&context).unwrap(),
        )),
    };
    return request;
}

async fn get_public_key() -> Result<PublicKeyResponse, String> {
    let request = ECDSAPublicKey {
        canister_id: None,
        derivation_path: vec![],
        key_id: EcdsaKeyIds::TestKeyLocalDevelopment.to_key_id(),
    };
    let (res_public_key,): (ECDSAPublicKeyReply,) =
        ic_cdk::call(mgmt_canister_id(), "ecdsa_public_key", (request,))
            .await
            .map_err(|e| format!("ecdsa_public_key failed {}", e.1))?;

    Ok(PublicKeyResponse {
        public_key: res_public_key.public_key.clone(),
        public_key_bs64: Engine::encode(&STANDARD, &res_public_key.public_key.clone()),
    })
}

async fn sign_with_ecdsa(digest: [u8; 32]) -> Vec<u8> {
    let request = SignWithECDSA {
        message_hash: sha256(digest).to_vec(),
        derivation_path: vec![],
        key_id: EcdsaKeyIds::TestKeyLocalDevelopment.to_key_id(),
    };

    let cycles = get_req_cycles();
    let (response,): (SignWithECDSAReply,) = ic_cdk::api::call::call_with_payment128(
        mgmt_canister_id(),
        "sign_with_ecdsa",
        (request,),
        cycles,
    )
    .await
    .map_err(|e| format!("sign_with_ecdsa failed {}", e.1))
    .unwrap();

    return response.signature;
}

async fn execute_tx_block_sui_rpc(signature: String, tx_bytes: String) -> Result<String, String> {
    let context = Context {
        bucket_start_time_index: 0,
        closing_price_index: 4,
    };

    let request_json: String = format!("{{\"jsonrpc\": \"2.0\",\"id\": 1,\"method\": \"sui_executeTransactionBlock\",\"params\": [\"{}\",[\"{}\"],null,null]}}", tx_bytes, signature).to_string();
    let sui_rpc_url = get_sui_rpc_url();
    let effective_size_estimate = get_effective_size_estimate();
    let cycles = get_req_cycles();
    let request = CanisterHttpRequestArgument {
        url: sui_rpc_url,
        max_response_bytes: Some(effective_size_estimate),
        method: HttpMethod::POST,
        headers: vec![HttpHeader {
            name: "Content-Type".to_string(),
            value: "application/json".to_string(),
        }],
        body: Some(request_json.to_string().into_bytes()),
        transform: Some(TransformContext::from_name(
            "cleanup_response".to_owned(),
            serde_json::to_vec(&context).unwrap(),
        )),
    };

    match http_request(request, cycles).await {
        Ok((response,)) => {
            let str_body = String::from_utf8(response.body.clone())
                .expect("Transformed response is not UTF-8 encoded.");
            ic_cdk::api::print(format!("{:?}", str_body));
            let resp = serde_json::from_slice::<ExecuteTxBlockResponse>(&response.body)
                .map_err(|e| format!("Error: {}", e.to_string()));

            if resp.is_err() {
                log!(INFO, "{:?}", resp.err());
                return Err("Failed to get tx digest".to_string());
            }

            return Ok(resp.unwrap().result.digest);
        }

        Err((r, m)) => {
            log!(
                INFO,
                "The http_request resulted into error. RejectionCode: {r:?}, Error: {m}"
            );
            return Err(m.to_string());
        }
    };
}

fn get_effective_size_estimate() -> u64 {
    let response_size_estimate = ResponseSizeEstimate::new(256);
    const HEADER_SIZE_LIMIT: u64 = 2 * 1024;
    let effective_size_estimate = response_size_estimate.get() + HEADER_SIZE_LIMIT;
    return effective_size_estimate;
}

fn get_req_cycles() -> u128 {
    let effective_size_estimate = get_effective_size_estimate();
    // Details of the values used in the following lines can be found here:
    // https://internetcomputer.org/docs/current/developer-docs/production/computation-and-storage-costs
    let base_cycles = 400_000_000u128 + 100_000u128 * (2 * effective_size_estimate as u128);

    const BASE_SUBNET_SIZE: u128 = 13;
    const SUBNET_SIZE: u128 = 34;
    let cycles = base_cycles * SUBNET_SIZE / BASE_SUBNET_SIZE;
    return cycles;
}

fn sha256(input: [u8; 32]) -> [u8; 32] {
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(input);
    hasher.finalize().into()
}

fn get_sui_rpc_url() -> String {
    let is_local = self::get(constants::IS_LOCAL_KEY.to_string()).unwrap();

    match is_local.as_str() {
        "true" => SUI_TESTNET_RPC_URL.to_string(),
        _ => SUI_MAINNET_RPC_URL.to_string(),
    }
}

fn mgmt_canister_id() -> Principal {
    let is_local = self::get(constants::IS_LOCAL_KEY.to_string()).unwrap();
    let ledger_id = self::get(constants::LOCAL_MGMT_PRINCIPAL_ID_KEY.to_string()).unwrap();

    match is_local.as_str() {
        "true" => Principal::from_str(&ledger_id).unwrap(),
        _ => Principal::management_canister(),
    }
}

fn get(key: String) -> Option<String> {
    MAP.with(|p| p.borrow().get(&KeyName(key)).map(|v| v.0))
}

fn insert(key: String, value: String) -> Option<String> {
    MAP.with(|p| p.borrow_mut().insert(KeyName(key), KeyValue(value)))
        .map(|v| v.0)
}

fn insert_mint_tx(key: String, value: String) -> Option<String> {
    MINTED_TRANSACTIONS
        .with(|p| p.borrow_mut().insert(KeyName(key), KeyValue(value)))
        .map(|v| v.0)
}

fn insert_withdraw_tx(key: String, value: String) -> Option<String> {
    FINALIZED_TRANSACTIONS
        .with(|p| p.borrow_mut().insert(KeyName(key), KeyValue(value)))
        .map(|v| v.0)
}

#[query]
fn get_minted_transactions() -> Vec<KeyValue> {
    MINTED_TRANSACTIONS.with(|transactions| {
        return transactions
            .borrow()
            .iter()
            .into_iter()
            .map(|(_, value)| value)
            .collect();
    })
}

#[query]
fn get_finalized_transactions() -> Vec<KeyValue> {
    FINALIZED_TRANSACTIONS.with(|transactions| {
        return transactions
            .borrow()
            .iter()
            .into_iter()
            .map(|(_, value)| value)
            .collect();
    })
}
