
use std::str::FromStr;
use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod, HttpResponse, TransformArgs,
    TransformContext,
};
use ic_cdk_macros::{self, query};
use serde::Serialize;
use serde_json::{self};
use ic_canister_log::log;
mod logs;
use crate::logs::INFO;
use candid::{CandidType, Deserialize};
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc1::transfer::NumTokens;
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

#[ic_cdk::update]
async fn mint() -> String {
    use ic_principal::Principal;
    use icrc_ledger_client::{CdkRuntime, ICRC1Client};
    use icrc_ledger_types::icrc1::transfer::TransferArg;
    use icrc_ledger_types::icrc1::account::Account;

    let url = "https://fullnode.testnet.sui.io:443";

    let json_string : String = "{\"jsonrpc\": \"2.0\",\"id\": 1,\"method\": \"suix_queryEvents\",\"params\":[{\"MoveModule\": {\"package\":\"0x44720817255b799b5c23d722568e850d07db51bf52b9c0425b3b16a1fe5f21a0\",\"module\": \"ckSuiHelper\"}},null,null,false]}".to_string();


    let json_utf8: Vec<u8> = json_string.into_bytes();
    let request_body: Option<Vec<u8>> = Some(json_utf8);


    let context = Context {
        bucket_start_time_index: 0,
        closing_price_index: 4,
    };

    let request = CanisterHttpRequestArgument {
        url: url.to_string(),
        max_response_bytes: None, //optional for request
        method: HttpMethod::POST,
        headers: vec![HttpHeader {
            name: "Content-Type".to_string(),
            value: "application/json".to_string(),
        }],
        body: request_body,
        transform: Some(TransformContext::new(transform, serde_json::to_vec(&context).unwrap())),
    };

    match http_request(request).await {
      
        Ok((response,)) => {

        let trasnaction = serde_json::from_slice::<receipt::Root>(&response.body)
            .map_err(|e| format!("Error: {}", e.to_string()));
        let principal: Principal = Principal::from_str(&trasnaction.clone().unwrap().result.data[0].parsed_json.principal_address).unwrap();
        let value: NumTokens  = NumTokens::from_str(&trasnaction.clone().unwrap().result.data[0].parsed_json.value).unwrap();
    
            let ledger_canister_id: Principal = Principal::from_text("mxzaz-hqaaa-aaaar-qaada-cai").unwrap();

            let client = ICRC1Client {
                runtime: CdkRuntime,
                ledger_canister_id,
            };
            let to: Account = Account {
                owner: principal,
                subaccount: None,
            };
            let block_index = match client
                .transfer( TransferArg {
                    from_subaccount: None,
                    to,
                    fee: None,
                    created_at_time: None,
                    memo: None,
                    amount: value,
     
                })
                .await
                {
                    Ok(Ok(block_index)) => block_index.to_string(),
                    Ok(Err(err)) => {
                        err.to_string() // Change this line to return only the error message as a string
                    }
                    Err(err) => {
                        log!(
                            INFO,
                            "Failed to send a message to the ledger ({ledger_canister_id}): {err:?}"
                        );
                        "error".to_string()
                    }
                };
           
                log!(
                    INFO,
                    "Block index ({block_index})" );
                
               

            let str_body = String::from_utf8(response.body)
                .expect("Transformed response is not UTF-8 encoded.");
            ic_cdk::api::print(format!("{:?}", str_body));

            let result: String = format!(
                "{}. See more info of the request sent at: {}/inspect",
                str_body, url
            );
            result
        }
        Err((r, m)) => {
            let message =
                format!("The http_request resulted into error. RejectionCode: {r:?}, Error: {m}");
            message
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


