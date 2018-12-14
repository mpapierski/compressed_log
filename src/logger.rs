//! A logger that stores all log information in a compressed
//! buffer, and once the buffer is full it executes a rotation
//! strategy.

use crate::format::{BinaryLogFormat, LogFormat};
use failure::Error;
use log::{Level, Log, Metadata, Record};
use lz4::{Encoder, EncoderBuilder};
use std::cell::RefCell;
use std::sync::Mutex;

/// A compressed logger structure.
struct Logger {
    level: Level,
    encoder: Mutex<RefCell<Encoder<Vec<u8>>>>,
}

impl Logger {
    pub fn new_encoder() -> Result<Encoder<Vec<u8>>, Error> {
        // This is the empty buffer that needs to be passed as output of LZ4
        let buffer = Vec::<u8>::new();
        // TODO: This should be more configurable
        Ok(EncoderBuilder::new().level(4).build(buffer)?)
    }

    // Create new Logger with given logging level
    pub fn with_level(level: Level) -> Result<Self, Error> {
        // Create new LZ4 encoder which may potentially fail.
        let encoder = Logger::new_encoder()?;
        // Return the logger instance
        Ok(Self {
            level,
            encoder: Mutex::new(RefCell::new(encoder)),
        })
    }

    /// Gets the length of compressed buffer.
    ///
    /// TODO: This function is probably unnecessary in production.
    fn len(&self) -> Result<usize, Error> {
        let encoder = self.encoder.lock().unwrap();
        let encoder = encoder.borrow();
        Ok(encoder.writer().len())
    }

    /// Rotates the internal LZ4 buffer and returns the compressed data
    /// buffer.
    fn rotate(&self) -> Result<Vec<u8>, Error> {
        // Acquire encoder lock
        let encoder = self.encoder.lock().unwrap();
        // Prepare new LZ4 encoder
        let new_encoder = Logger::new_encoder()?;
        // Retrieve the old encoder by swapping it with the new one
        let old_encoder = encoder.replace(new_encoder);
        // Finish up the last bits of LZ4 stream and get the writer
        let (writer, result) = old_encoder.finish();
        // Check for any compression errors at the last step
        let _ = result?;
        // Return the data buffer
        Ok(writer)
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
    assert!(logger.len().unwrap() > 0usize);

    let data = logger.rotate().unwrap();
    assert!(logger.len().unwrap() != data.len());
}
