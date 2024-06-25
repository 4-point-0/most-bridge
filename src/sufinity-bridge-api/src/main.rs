use actix_settings::{ApplySettings as _, BasicSettings, Mode};
use actix_web::{
    body::BoxBody,
    error::ErrorUnauthorized,
    http::header::{self, ContentType},
    middleware::{Compress, Condition, Logger},
    post,
    web::{self},
    App, Error as ActixError, HttpRequest, HttpResponse, HttpServer, Responder,
};
use base64::{self, engine::general_purpose::STANDARD, Engine};
use constants::GAS_BUDGET;
use fastcrypto::{
    encoding::{Base64, Encoding},
    hash::HashFunction,
};
use models::{PaySuiRequest, PaySuiResponse, TxDigestRequest};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use serde::{Deserialize, Serialize};
use shared_crypto::intent::{Intent, IntentMessage};
use sui_types::transaction::TransactionData;
mod constants;
mod models;
use actix_cors::Cors;

#[derive(Serialize, Clone)]
struct Reply {
    pub digest: String,
    pub tx_bytes: String,
}

#[derive(Deserialize, Clone)]
struct ApplicationSettings {
    pub signer: String,
    pub gas_object_id: String,
    pub mainnet_url: String,
    pub testnet_url: String,
    pub address_ssl: String,
    pub api_key: String,
    pub origin_url: String,
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
async fn tx_digest(
    _req: HttpRequest,
    dto: web::Json<TxDigestRequest>,
    settings: web::Data<BasicSettings<ApplicationSettings>>,
) -> Result<Reply, ActixError> {
    let api_key_header = _req.headers().get("X-API-Key").unwrap().to_str().unwrap();

    if api_key_header != settings.application.api_key.to_string() {
        return Err(ErrorUnauthorized("Unathorized access"));
    }
    match transfer_sui(dto.clone(), settings).await {
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

async fn transfer_sui(
    dto: TxDigestRequest,
    settings: web::Data<BasicSettings<ApplicationSettings>>,
) -> Result<std::string::String, reqwest::Error> {
    let model: PaySuiRequest = PaySuiRequest {
        jsonrpc: "2.0".to_string(),
        id: 1,
        method: "unsafe_transferSui".to_string(),
        params: vec![
            settings.application.signer.to_string(),        //signer
            settings.application.gas_object_id.to_string(), //sui_object_id
            GAS_BUDGET.to_string(),                         //gas budget
            dto.recipient,                                  //recipient
            dto.amount,                                     //amount
        ],
    };

    let response = reqwest::Client::new()
        .post(match settings.actix.mode {
            Mode::Development => settings.application.testnet_url.to_string(),
            Mode::Production => settings.application.mainnet_url.to_string(),
        })
        .json(&model)
        .send()
        .await?;

    let resp = response.json::<PaySuiResponse>().await?;

    return Ok(resp.result.tx_bytes);
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let settings =
        BasicSettings::<ApplicationSettings>::parse_toml("src/sufinity-bridge-api/config.toml")
            .expect("Failed to parse `Settings` from config.toml");

    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file(settings.clone().actix.tls.private_key, SslFiletype::PEM)
        .unwrap();
    builder
        .set_certificate_chain_file(settings.clone().actix.tls.certificate)
        .unwrap();

    init_logger(&settings);

    HttpServer::new({
        let settings = settings.clone();
        move || {
            App::new()
                .wrap(Condition::new(
                    settings.actix.enable_compression,
                    Compress::default(),
                ))
                .wrap(Logger::default())
                .app_data(web::Data::new(settings.clone()))
                .wrap(
                    Cors::default()
                        // add specific origin to allowed origin list
                        .allowed_origin(settings.application.origin_url.as_str())
                        // set allowed methods list
                        .allowed_methods(vec!["GET", "POST"])
                        // set allowed request header list
                        .allowed_headers(&[header::AUTHORIZATION, header::ACCEPT])
                        // add header to allowed list
                        .allowed_header(header::CONTENT_TYPE)
                        // set list of headers that are safe to expose
                        .expose_headers(&[header::CONTENT_DISPOSITION])
                        // allow cURL/HTTPie from working without providing Origin headers
                        .block_on_origin_mismatch(false)
                        // set preflight cache TTL
                        .max_age(3600),
                )
                .service(tx_digest)
        }
    })
    .apply_settings(&settings)
    .bind_openssl(settings.application.address_ssl, builder)?
    .run()
    .await
}

fn init_logger(settings: &BasicSettings<ApplicationSettings>) {
    if !settings.actix.enable_log {
        return;
    }

    std::env::set_var(
        "RUST_LOG",
        match settings.actix.mode {
            Mode::Development => "actix_web=debug",
            Mode::Production => "actix_web=info",
        },
    );

    std::env::set_var("RUST_BACKTRACE", "1");

    env_logger::init();
}
