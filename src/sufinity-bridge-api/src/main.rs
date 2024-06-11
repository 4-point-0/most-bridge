use actix_web::{
    body::BoxBody, get, http::header::ContentType, http::KeepAlive, App, HttpRequest, HttpResponse,
    HttpServer, Responder, Result,
};
use base64::{self, engine::general_purpose::STANDARD, Engine};
use fastcrypto::{
    encoding::{Base64, Encoding},
    hash::HashFunction,
};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use serde::Serialize;
use shared_crypto::intent::{Intent, IntentMessage};
use sui_types::transaction::TransactionData;

#[derive(Serialize)]
struct Reply {
    pub digest: String,
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
#[get("/tx-digest")]
async fn index(tx_bytes: String) -> Result<Reply> {
    let decoded = Engine::decode(&STANDARD, tx_bytes).unwrap();
    let tx_data: TransactionData = bcs::from_bytes(&decoded).unwrap();

    let intent_msg = IntentMessage::new(Intent::sui_transaction(), tx_data);

    let raw_intent_msg: Vec<u8> = match bcs::to_bytes(&intent_msg) {
        Ok(bytes) => bytes,
        Err(err) => panic!("Failed to serialize intent message: {}", err.to_string()),
    };

    let mut hasher = sui_types::crypto::DefaultHash::default();
    hasher.update(raw_intent_msg);
    let digest = hasher.finalize().digest;
    let encoded = Base64::encode(digest);

    return Ok(Reply { digest: encoded });
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("cert/key.pem", SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file("cert/cert.pem").unwrap();

    HttpServer::new(|| App::new().service(index))
        .keep_alive(KeepAlive::Os)
        .bind_openssl("[::1]:8080", builder)?
        .run()
        .await
}
