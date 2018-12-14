//! A logger that stores all log information in a compressed
//! buffer, and once the buffer is full it executes a rotation
//! strategy.

use crate::format::{BinaryLogFormat, LogFormat};
use failure::Error;
use log::{Level, Log, Metadata, Record};
use lz4::{Encoder, EncoderBuilder};
use std::cell::RefCell;
use std::sync::Mutex;

struct Logger {
    level: Level,
    encoder: Mutex<RefCell<Encoder<Vec<u8>>>>,
}

impl Logger {
    pub fn with_level(level: Level) -> Result<Self, Error> {
        let buffer = Vec::<u8>::new();
        let encoder = EncoderBuilder::new().level(4).build(buffer)?;
        Ok(Self {
            level,
            encoder: Mutex::new(RefCell::new(encoder)),
        })
    }

    fn size(&self) -> Result<usize, Error> {
        let encoder = self.encoder.lock().unwrap();
        let encoder = encoder.borrow();
        Ok(encoder.writer().len())
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
            let encoder = self.encoder.lock().expect("Unable to acquire buffer lock");
            // let mut encoder = *encoder.borrow_mut();
            // Serialize binary log record into the output buffer

            log_format
                .serialize(&mut *encoder.borrow_mut())
                .expect("Unable to serialize a log record into the compressed stream");
        }
    }

    fn flush(&self) {}
}

/// Initializes the global logger with a Logger instance with
/// `max_log_level` set to a specific log level.
///
pub fn init_with_level(level: Level) -> Result<(), Error> {
    let logger = Logger::with_level(level)?;
    log::set_boxed_logger(Box::new(logger))?;
    log::set_max_level(level.to_level_filter());
    Ok(())
}

/// Initializes the global logger with a Logger instance with
/// `max_log_level` set to `LogLevel::Trace`.
///
pub fn init() -> Result<(), Error> {
    init_with_level(Level::Trace)
}

#[test]
fn logger() {
    use log::{Level, Record};
    let record = Record::builder()
        .args(format_args!("Error!"))
        .level(Level::Error)
        .target("myApp")
        .file(Some("server.rs"))
        .line(Some(144))
        .module_path(Some("server"))
        .build();

    let logger = Logger::with_level(Level::Trace).unwrap();
    logger.log(&record);
    assert!(logger.size().unwrap() > 0usize);
}
