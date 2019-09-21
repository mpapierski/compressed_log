use crate::logger::{CompressedLogs, PlaintextLogs};
use actix::Arbiter;
use actix_web::client;
use actix_web::http::header;
use futures::Future;
use std::time::Duration;

static TIMEOUT: Duration = Duration::from_secs(1);

pub fn plaintext_log_upload(msg: PlaintextLogs, url: String) {
    debug_eprintln!("post? {}", url);
    let res = client::post(&url)
        .header(header::CONTENT_TYPE, "application/json")
        .json(msg)
        .unwrap()
        .send()
        .timeout(TIMEOUT)
        .then(|response| {
            debug_eprintln!("response {:?}", response);
            let _response = response;
            Ok(())
        });
    Arbiter::spawn(res);
}

pub fn compressed_log_upload(msg: CompressedLogs, url: String) {
    debug_eprintln!("compressed post? {}", url);
    let res = client::post(&url)
        .header(header::CONTENT_TYPE, "application/json")
        .json(msg)
        .unwrap()
        .send()
        .timeout(TIMEOUT)
        .then(|response| {
            debug_eprintln!("response {:?}", response);
            let _response = response;
            Ok(())
        });
    Arbiter::spawn(res);
}
