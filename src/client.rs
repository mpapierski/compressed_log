use crate::{
    builder::LoggerError,
    logger::{CompressedLogs, PlaintextLogs},
};
use actix_web::client::Client;
use actix_web::http::header;
use std::time::Duration;

static TIMEOUT: Duration = Duration::from_secs(5);

pub async fn plaintext_log_upload(msg: PlaintextLogs, url: String) -> Result<(), LoggerError> {
    debug_eprintln!("post? to {} with {} bytes", url, msg.logs.len());
    // an Actix web client instance with the default setting. the main gotcha
    // to check here is maximum payload size if you want to go really big
    let client = Client::default();
    let res = client
        .post(&url)
        .header(header::CONTENT_TYPE, "application/json")
        .timeout(TIMEOUT)
        .send_json(&msg)
        .await;
    debug_eprintln!("response {:?}", res);
    match res {
        Ok(_) => Ok(()),
        Err(e) => Err(LoggerError::ConnectionFailure(e)),
    }
}

pub async fn compressed_log_upload(msg: CompressedLogs, url: String) -> Result<(), LoggerError> {
    debug_eprintln!(
        "compressed post? to {} with {} bytes",
        url,
        msg.compressed_plaintext_logs.len()
    );
    // an Actix web client instance with the default setting. the main gotcha
    // to check here is maximum payload size if you want to go really big
    let client = Client::default();
    let res = client
        .post(&url)
        .header(header::CONTENT_TYPE, "application/json")
        .timeout(TIMEOUT)
        .send_json(&msg)
        .await;
    debug_eprintln!("response {:?}", res);
    match res {
        Ok(_) => Ok(()),
        Err(e) => Err(LoggerError::ConnectionFailure(e)),
    }
}
