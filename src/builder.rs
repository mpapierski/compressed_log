use crate::compression::Compression;
use crate::formatter::{default_formatter, Formatter};
use crate::logger::Logger;
use awc::error::PayloadError;
use awc::error::SendRequestError;
use log::Level;
use std::cell::RefCell;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum LoggerError {
    UploadFailure(PayloadError),
    ConnectionFailure(SendRequestError),
    IoError(io::Error),
}

impl fmt::Display for LoggerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LoggerError::UploadFailure(e) => write!(f, "Actix Upload Failure {:?}", e),
            LoggerError::ConnectionFailure(e) => write!(f, "Actix Connection Failure {:?}", e),
            LoggerError::IoError(e) => write!(f, "IoError {:?}", e),
        }
    }
}

impl From<io::Error> for LoggerError {
    fn from(error: io::Error) -> Self {
        LoggerError::IoError(error)
    }
}

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
            compression: Compression::None,
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
        let _ = self.format.replace(format);
        self
    }
    pub fn build(&self) -> Result<Logger, LoggerError> {
        debug_eprintln!("Building compressed logger");
        assert!(
            self.sink_url.is_some(),
            "Unable to create Logger instance without sink url"
        );
        let sink_url = self.sink_url.as_ref().unwrap().clone();

        // Extract inner formatter by swapping it
        let formatter = self.format.replace(Box::new(default_formatter));

        Logger::with_level(
            self.level,
            self.compression,
            self.threshold,
            sink_url,
            formatter,
        )
    }
}
