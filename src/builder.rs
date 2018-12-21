use crate::client::{Connect, LogClient};
use crate::logger::Logger;
use crate::lz4::Compression;
use actix::{Actor, Arbiter, Supervisor};
use failure::Error;
use futures::future::Future;
use log::Level;

pub struct LoggerBuilder {
    level: Level,
    compression: Compression,
    sink_url: Option<String>,
    threshold: usize,
}

impl LoggerBuilder {
    pub fn new() -> LoggerBuilder {
        LoggerBuilder {
            /// Log all messages by default
            level: Level::Trace,
            /// Default is supposed to be low to provide fast on the fly compression
            compression: Compression::Fast,
            sink_url: None,
            /// Default threshold is about ~32KB of compressed data
            threshold: 32000usize,
        }
    }
    pub fn set_level(&mut self, level: Level) -> &mut Self {
        self.level = level;
        self
    }
    /// Sets compression level
    pub fn set_compression_level<'a>(
        &'a mut self,
        compression: Compression,
    ) -> &'a mut LoggerBuilder {
        self.compression = compression;
        self
    }
    pub fn set_sink_url(&mut self, url: &str) -> &mut Self {
        self.sink_url = Some(url.to_string());
        self
    }
    /// Sets the threshold in bytes
    pub fn set_threshold(&mut self, threshold: usize) -> &mut Self {
        self.threshold = threshold;
        self
    }
    pub fn build(&self) -> Result<Logger, Error> {
        ensure!(
            self.sink_url.is_some(),
            "Unable to create Logger instance without sink url"
        );

        let addr = {
            let addr = Supervisor::start(|ctx| LogClient::default());
            let url = self.sink_url.as_ref().unwrap().clone();
            Arbiter::spawn(addr.send(Connect(url)).map_err(|e| {
                eprintln!("Unable to send data {}", e);
                ()
            }));
            addr
        };
        Logger::new(self.level, self.compression, self.threshold, addr)
    }
}
