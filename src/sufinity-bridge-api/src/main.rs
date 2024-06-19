use actix_web::{
    body::BoxBody,
    http::{header::ContentType, KeepAlive},
    middleware::Logger,
    post, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder,
};
use base64::{self, engine::general_purpose::STANDARD, Engine};
use constants::{GAS_OBJECT_ID, SIGNER};
use fastcrypto::{
    encoding::{Base64, Encoding},
    hash::HashFunction,
};
use models::{PaySuiRequest, PaySuiResponse, TxDigestRequest};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use serde::Serialize;
use shared_crypto::intent::{Intent, IntentMessage};
use sui_types::transaction::TransactionData;
mod constants;
mod models;

#[derive(Serialize, Clone)]
struct Reply {
    pub digest: String,
    pub tx_bytes: String,
}

impl Responder for Reply {
    type Body = BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        let body = serde_json::to_string(&self).unwrap();

        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(body)
    }
}

#[post("/tx-digest")]
async fn tx_digest(dto: web::Json<TxDigestRequest>) -> Result<Reply, Error> {
    match transfer_sui(dto.clone()).await {
        Ok(tx_bytes) => {
            let decoded = Engine::decode(&STANDARD, tx_bytes.clone()).unwrap();
            let tx_data: TransactionData = bcs::from_bytes(&decoded).unwrap();

            let intent_msg = IntentMessage::new(Intent::sui_transaction(), tx_data);

            let raw_intent_msg: Vec<u8> = match bcs::to_bytes(&intent_msg) {
                Ok(bytes) => bytes,
                Err(err) => panic!("Failed to serialize intent message: {}", err.to_string()),
            };

            let mut hasher = sui_types::crypto::DefaultHash::default();
            hasher.update(raw_intent_msg);
            let dig = hasher.finalize().digest;
            let digest = Base64::encode(dig);

            return Ok(Reply {
                digest,
                tx_bytes: tx_bytes.clone(),
            });
        }
        Err(err) => panic!("Failed to get tx digest: {}", err.to_string()),
    }
}

async fn transfer_sui(dto: TxDigestRequest) -> Result<std::string::String, reqwest::Error> {
    let model: PaySuiRequest = PaySuiRequest {
        jsonrpc: "2.0".to_string(),
        id: 1,
        method: "unsafe_transferSui".to_string(),
        params: vec![
            SIGNER.to_string(),        //signer
            GAS_OBJECT_ID.to_string(), //sui_object_id
            "100000000".to_string(),   //gas budget
            dto.recipient,             //recipient
            dto.amount,                //amount
        ],
    };

    let response = reqwest::Client::new()
        .post("https://fullnode.testnet.sui.io/")
        .json(&model)
        .send()
        .await?;

    let resp = response.json::<PaySuiResponse>().await?;

    return Ok(resp.result.tx_bytes);
}

// async fn gas_price() -> Result<String, reqwest::Error> {
//     let model = ReferenceGasPriceRequest {
//         jsonrpc: "2.0".to_string(),
//         id: 1,
//         method: "suix_getReferenceGasPrice".to_string(),
//         params: vec![],
//     };

//     let response = reqwest::Client::new()
//         .post("https://fullnode.testnet.sui.io/")
//         .json(&model)
//         .send()
//         .await?;

//     let resp = response.json::<ReferenceGasPriceResponse>().await?;
//     info!("Gas price response {}", resp.result);
//     Ok(resp.result)
// }

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "info");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("src/sufinity-bridge-api/cert/key.pem", SslFiletype::PEM)
        .unwrap();
    builder
        .set_certificate_chain_file("src/sufinity-bridge-api/cert/cert.pem")
        .unwrap();

    HttpServer::new(|| {
        let logger = Logger::default();
        App::new().wrap(logger).service(tx_digest)
    })
    .keep_alive(KeepAlive::Os)
    .bind_openssl("[::1]:8080", builder)?
    .run()
    .await
}
