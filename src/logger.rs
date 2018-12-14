//! A logger that stores all log information in a compressed
//! buffer, and once the buffer is full it executes a rotation
//! strategy.

use crate::format::{BinaryLogFormat, LogFormat};
use log::{Level, Log, Metadata, Record, SetLoggerError};
use std::cell::RefCell;
use std::sync::Mutex;

struct Logger {
    level: Level,
    buffer: Mutex<RefCell<Vec<u8>>>,
}
impl Logger {
    pub fn new(level: Level) -> Self {
        Self {
            level,
            buffer: Mutex::new(RefCell::new(Vec::new())),
        }
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            // Prepare a binary log record formatter
            let log_format = BinaryLogFormat::from_record(&record);
            // Acquire buffer instance
            let buffer = self.buffer.lock().expect("Unable to acquire buffer lock");
            // Serialize binary log record into the output buffer
            log_format.serialize(&mut *buffer.borrow_mut()).unwrap();
        }
    }

    fn flush(&self) {}
}

/// Initializes the global logger with a Logger instance with
/// `max_log_level` set to a specific log level.
///
pub fn init_with_level(level: Level) -> Result<(), SetLoggerError> {
    let logger = Logger::new(level);
    log::set_boxed_logger(Box::new(logger))?;
    log::set_max_level(level.to_level_filter());
    Ok(())
}

/// Initializes the global logger with a Logger instance with
/// `max_log_level` set to `LogLevel::Trace`.
///
pub fn init() -> Result<(), SetLoggerError> {
    init_with_level(Level::Trace)
}
