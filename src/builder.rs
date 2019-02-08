use crate::client::{Connect, LogClient};
use crate::formatter::{default_formatter, Formatter};
use crate::logger::Logger;
use crate::lz4::Compression;
use actix::Supervisor;
use failure::Error;
use log::Level;
use std::cell::RefCell;

pub struct LoggerBuilder {
    level: Level,
    compression: Compression,
    sink_url: Option<String>,
    threshold: usize,
    format: RefCell<Box<Formatter>>,
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
            format: RefCell::new(Box::new(default_formatter)),
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
    pub fn set_format(&mut self, format: Box<Formatter>) -> &mut Self {
        self.format.replace(format);
        self
    }
    pub fn build(&self) -> Result<Logger, Error> {
        ensure!(
            self.sink_url.is_some(),
            "Unable to create Logger instance without sink url"
        );

        let addr = {
            let addr = Supervisor::start(|_ctx| LogClient::default());
            let url = self
                .sink_url
                .as_ref()
                .expect("Unable to obtain sink url")
                .clone();
            addr.try_send(Connect(url))?;
            addr
        };

        // Extract inner formatter by swapping it
        let formatter = self.format.replace(Box::new(default_formatter));

        Logger::with_level(
            self.level,
            self.compression,
            self.threshold,
            addr,
            formatter,
        )
    }
}
