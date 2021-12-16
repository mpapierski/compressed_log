//! A logger that stores all log information in a compressed
//! buffer, and once the buffer is full it executes a rotation
//! strategy.

use crate::builder::LoggerError;
use crate::client::compressed_log_upload;
use crate::client::plaintext_log_upload;
use crate::compression::Compression;
use crate::compression::Encoder;
use crate::compression::FinishValue;
use actix::System;
use lazy_static::lazy_static;
use log::{Level, Log, Metadata, Record};
use std::cell::RefCell;
use std::sync::Mutex;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};

lazy_static! {
    /// Timer that sends logs over to log_sink when the timer expires. Timeout is measured by LOG_TIMEOUT
    pub static ref TIMER: Arc<RwLock<Instant>> = Arc::new(RwLock::new(Instant::now()));
}

//Time after which logs are sent over to the sink, regardless of whether the buffer is filled or not
const LOG_TIMEOUT: Duration = Duration::from_secs(10 * 60);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlaintextLogs {
    /// A vector of log strings
    pub logs: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CompressedLogs {
    /// literally a compressed json serialized version of the plaintext logs struct above
    pub compressed_plaintext_logs: Vec<u8>,
}

/// A compressed logger structure.
pub struct Logger {
    level: Level,
    compression: Compression,
    threshold: usize,
    encoder: Mutex<RefCell<Encoder>>,
    sink_url: String,
    format: Box<dyn Fn(&Record) -> String + Sync + Send>,
}

impl Logger {
    pub fn new_encoder(compression: Compression) -> Result<Encoder, LoggerError> {
        // This is the empty buffer that needs to be passed as output of LZ4
        Ok(Encoder::new(compression))
    }

    // Create new Logger with given logging level
    pub fn with_level(
        level: Level,
        compression: Compression,
        threshold: usize,
        sink_url: String,
        format: Box<dyn Fn(&Record) -> String + Sync + Send>,
    ) -> Result<Self, LoggerError> {
        // Create new LZ4 encoder which may potentially fail.
        let encoder = Logger::new_encoder(compression)?;
        // Return the logger instance
        Ok(Self {
            level,
            compression,
            threshold,
            encoder: Mutex::new(RefCell::new(encoder)),
            sink_url,
            format,
        })
    }
    /// Rotates the internal compressed buffer and returns the compressed data
    /// buffer.
    fn rotate(&self, encoder: &RefCell<Encoder>) -> Result<FinishValue, LoggerError> {
        // Prepare new compressed encoder
        let new_encoder = Logger::new_encoder(self.compression)?;
        // Retrieve the old encoder by swapping it with the new one
        let old_encoder = encoder.replace(new_encoder);
        // Finish up the last bits the stream stream and get the writer
        let res = old_encoder.finish()?;
        // Return the data buffer
        Ok(res)
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        debug_eprintln!("Drop handler called!");
        // Unconditional rotation of log
        let encoder = self.encoder.lock().expect("Unable to acquire buffer lock");
        let data = self.rotate(&encoder).expect("Unable to rotate the buffer");
        match data {
            FinishValue::Compressed(ref c) => {
                if c.compressed_plaintext_logs.is_empty() {
                    return;
                }
            }
            FinishValue::Uncompressed(ref c) => {
                if c.logs.is_empty() {
                    return;
                }
            }
        }
        let url = self.sink_url.clone();
        // Send a chunk of data using the connection
        upload_logs(url, data);
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            // Acquire buffer instance
            let encoder = self.encoder.lock().expect("Unable to acquire encoder lock");
            // Serialize binary log record into the output buffer
            let mut log_string = (self.format)(record);

            // Rotate the buffer based on a threshold
            let current_size = { encoder.borrow_mut().uncompressed_bytes() };

            let mut log_len = log_string.as_bytes().len();

            // Error case when a single log line is greater than the entire buffer size. In this case we set the log to an error string.
            if log_len > self.threshold {
                let error_str = format!("Single log line greater than log threshold of {}, please reduce log size.\n This logs starts with {:?}", self.threshold, {
                    let mut log_clone = log_string.clone();
                    log_clone.truncate(1000);
                    log_clone
                });
                debug_eprintln!("{}", error_str);
                log_string = error_str;
                log_len = log_string.as_bytes().len();
            }

            if (current_size + log_len < self.threshold)
                && (Instant::now() - *TIMER.read().unwrap() < LOG_TIMEOUT)
            {
                encoder.borrow_mut().add_line(log_string.clone());
                debug_eprintln!(
                    "Buffer {} of {} bytes",
                    current_size + log_len,
                    self.threshold
                );
                debug_eprintln!("First 5000 bytes of Line {:?}", {
                    let mut log_clone = log_string;
                    log_clone.truncate(5000);
                    log_clone
                });
                // Compressed log didn't hit the size threshold
                // if timeout is hit, send logs anyway
                return;
            }

            debug_eprintln!("Size greater than threshold or timeout hit, sending logs");

            // Save the memory buffer using already acquired encoder instance.
            // With this approach it wont require us to manually drop a lock on encoder,
            // just to acquire it again inside rotate() function.

            let data = self.rotate(&encoder).expect("Unable to rotate the buffer");

            let url = self.sink_url.clone();

            // Send a chunk of data using the connection
            upload_logs(url, data);

            //Add the current log to new empty buffer
            drop(encoder);
            self.log(record);
        }
    }

    fn flush(&self) {
        let encoder = self.encoder.lock().expect("Unable to acquire encoder lock");
        let data = self.rotate(&encoder).expect("Unable to rotate the buffer");
        debug_eprintln!("Flush called, dropping logs!");

        let url = self.sink_url.clone();
        // Send a chunk of data using the connection
        upload_logs(url, data);
    }
}

fn upload_logs(url: String, data: FinishValue) {
    debug_eprintln!("Uploading logs");

    // create a new thread for the actix executor
    // to adopt in order to run the future to completion
    thread::spawn(|| {
        debug_eprintln!("thread spawned");

        let runner = System::new();
        runner.block_on(async move {
            match data {
                FinishValue::Compressed(c) => {
                    let _ = compressed_log_upload(c, url).await;
                }
                FinishValue::Uncompressed(c) => {
                    let _ = plaintext_log_upload(c, url).await;
                }
            }
            System::current().stop();
        });
    });

    // reset timer
    *TIMER.write().unwrap() = Instant::now();
}

#[test]
/// A simple test environment for compressed log
fn test_logging() {
    use super::*;
    use crate::builder::LoggerBuilder;
    use log::LevelFilter;
    let logging_url = "https://stats.altheamesh.com:9999/compressed_sink";
    let level = LevelFilter::Info;

    let logger = LoggerBuilder::default()
        .set_level(level.to_level().unwrap())
        .set_compression_level(Compression::Fast)
        .set_sink_url(logging_url)
        .set_format(Box::new(move |record: &Record| {
            format!("compressed-logger-tester! {}\n", record.args())
        }))
        .build();
    let logger = logger.unwrap();
    log::set_boxed_logger(Box::new(logger)).unwrap();
    log::set_max_level(level);
    println!(
        "Remote compressed logging enabled with target {}",
        logging_url
    );
    for _ in 0..100_000 {
        info!("test!")
    }
}
