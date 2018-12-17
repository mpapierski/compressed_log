//! A logger that stores all log information in a compressed
//! buffer, and once the buffer is full it executes a rotation
//! strategy.

use crate::format::{BinaryLogFormat, LogFormat};
use crate::lz4::{Compression, Encoder, EncoderBuilder, InMemoryEncoder};
use failure::Error;
use log::{Level, Log, Metadata, Record};
use std::cell::RefCell;
use std::io::Write;
use std::sync::Mutex;

/// A compressed logger structure.
pub struct Logger {
    level: Level,
    compression: Compression,
    encoder: Mutex<RefCell<InMemoryEncoder>>,
}

impl Logger {
    pub fn new_encoder(compression: &Compression) -> Result<Encoder<Vec<u8>>, Error> {
        // This is the empty buffer that needs to be passed as output of LZ4
        let buffer = Vec::<u8>::new();
        // TODO: This should be more configurable
        Ok(EncoderBuilder::new()
            .level(compression.to_level())
            .build(buffer)?)
    }

    // Create new Logger with given logging level
    pub fn new(level: Level, compression: Compression) -> Result<Self, Error> {
        // Create new LZ4 encoder which may potentially fail.
        let encoder = Logger::new_encoder(&compression)?;
        // Return the logger instance
        Ok(Self {
            level,
            compression,
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
    fn rotate(&self, encoder: &RefCell<InMemoryEncoder>) -> Result<Vec<u8>, Error> {
        // Prepare new LZ4 encoder
        let new_encoder = Logger::new_encoder(&self.compression)?;
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

impl Drop for Logger {
    fn drop(&mut self) {
        // Unconditional rotation of log
        let encoder = self.encoder.lock().expect("Unable to acquire buffer lock");
        let data = self.rotate(&encoder).expect("Unable to rotate the buffer");
        eprintln!("Data at the end {:?}", data);
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
            // Serialize binary log record into the output buffer
            log_format
                .serialize(&mut *encoder.borrow_mut())
                .expect("Unable to serialize a log record into the compressed stream");

            // Flush internal buffers every time so we wouldn't accidentally rotate
            // without consistent data.
            encoder.borrow_mut().flush().expect("Unable to flush");

            // Rotate the buffer based on a threshold
            let current_size = {
                let encoder = encoder.borrow();
                let writer = encoder.writer();
                writer.len()
            };
            if current_size >= 60_000 {
                eprintln!("Trying to rotate...");
                // Save the memory buffer using already acquired encoder instance.
                // With this approach it wouldn't require us to manually drop a lock on encodr,
                // just to acquire it again inside rotate() function.
                let data = self.rotate(&encoder).expect("Unable to rotate the buffer");
                eprintln!("Compressed log chunk size: {}", data.len());
            }
        }
    }

    fn flush(&self) {}
}

/// Initializes the global logger with a Logger instance with
/// `max_log_level` set to a specific log level.
///
pub fn init_with_level(level: Level) -> Result<(), Error> {
    let logger = Logger::new(level, Compression::Fast)?;
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

    let logger = Logger::new(Level::Trace, Compression::Fast).unwrap();
    logger.log(&record);
    assert!(logger.len().unwrap() > 0usize);
}
