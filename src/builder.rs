use crate::logger::Logger;
use crate::lz4::Compression;
use failure::Error;
use log::Level;

struct LoggerBuilder {
    level: Level,
    compression: Compression,
}

impl LoggerBuilder {
    pub fn new() -> LoggerBuilder {
        LoggerBuilder {
            /// Log all messages by default
            level: Level::Trace,
            /// Default is supposed to be low to provide fast on the fly compression
            compression: Compression::Fast,
        }
    }
    /// Sets compression level
    pub fn set_compression_level<'a>(
        &'a mut self,
        compression: Compression,
    ) -> &'a mut LoggerBuilder {
        self.compression = compression;
        self
    }
    pub fn build(&self) -> Result<Logger, Error> {
        Logger::new(self.level, self.compression)
    }
}
