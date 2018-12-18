use crate::client::LogClient;
use crate::logger::Logger;
use crate::lz4::Compression;
use failure::Error;
use log::Level;

pub struct LoggerBuilder {
    level: Level,
    compression: Compression,
    sink_url: Option<String>,
}

impl LoggerBuilder {
    pub fn new() -> LoggerBuilder {
        LoggerBuilder {
            /// Log all messages by default
            level: Level::Trace,
            /// Default is supposed to be low to provide fast on the fly compression
            compression: Compression::Fast,
            sink_url: None,
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
    pub fn build(&self) -> Result<Logger, Error> {
        ensure!(
            self.sink_url.is_some(),
            "Unable to create Logger instance without sink url"
        );
        Logger::new(
            self.level,
            self.compression,
            LogClient::connect(&self.sink_url.as_ref().unwrap()),
        )
    }
}
