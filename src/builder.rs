use crate::client::{Connect, LogClient};
use crate::logger::Logger;
use crate::lz4::Compression;
use actix::{Arbiter, Supervisor};
use chrono::{DateTime, Local};
use failure::Error;
use futures::future::Future;
use log::Level;
use log::Record;
use std::cell::RefCell;
use std::io;

pub struct LoggerBuilder {
    level: Level,
    compression: Compression,
    sink_url: Option<String>,
    threshold: usize,
    format: RefCell<Option<Box<Fn(&Record) -> String + Sync + Send>>>,
}

impl Default for LoggerBuilder {
    fn default() -> LoggerBuilder {
        LoggerBuilder {
            /// Log all messages by default
            level: Level::Trace,
            /// Default is supposed to be low to provide fast on the fly compression
            compression: Compression::Fast,
            sink_url: None,
            /// Default threshold is about ~32KB of compressed data
            threshold: 32000usize,
            /// Default format for backwards compatibility
            format: RefCell::new(None),
        }
    }
}

impl LoggerBuilder {
    pub fn set_level(&mut self, level: Level) -> &mut Self {
        self.level = level;
        self
    }
    /// Sets compression level
    pub fn set_compression_level(&mut self, compression: Compression) -> &mut LoggerBuilder {
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
    pub fn set_format(&mut self, format: Box<Fn(&Record) -> String + Sync + Send>) -> &mut Self {
        self.format.replace(Some(format));
        self
    }
    pub fn build(&self) -> Result<Logger, Error> {
        ensure!(
            self.sink_url.is_some(),
            "Unable to create Logger instance without sink url"
        );

        let addr = {
            let addr = Supervisor::start(|_ctx| LogClient::default());
            let url = self.sink_url.as_ref().unwrap().clone();
            Arbiter::spawn(addr.send(Connect(url)).map_err(|_e| {
                debug_eprintln!("Unable to send data {}", _e);
            }));
            addr
        };

        let format = match self.format.replace(None) {
            Some(f) => f,
            None => Box::new(|record: &Record| {
                let timestamp = Local::now();
                format!(
                    "{} {:<5} [{}] {}\n",
                    timestamp.format("%Y-%m-%d %H:%M:%S"),
                    record.level().to_string(),
                    record.module_path().unwrap_or_default(),
                    record.args()
                )
            }),
        };
        Logger::with_level(self.level, self.compression, self.threshold, addr, format)
    }
}
