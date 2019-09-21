//! A logger that stores all log information in a compressed
//! buffer, and once the buffer is full it executes a rotation
//! strategy.

use crate::client::compressed_log_upload;
use crate::client::plaintext_log_upload;
use crate::compression::Compression;
use crate::compression::Encoder;
use crate::compression::FinishValue;
use failure::Error;
use log::{Level, Log, Metadata, Record};
use std::cell::RefCell;
use std::sync::Mutex;

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
    pub fn new_encoder(compression: Compression) -> Result<Encoder, Error> {
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
    ) -> Result<Self, Error> {
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
    fn rotate(&self, encoder: &RefCell<Encoder>) -> Result<FinishValue, Error> {
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
        // Send a chunk of data using the connection
        match data {
            FinishValue::Compressed(c) => {
                compressed_log_upload(c, self.sink_url.clone());
            }
            FinishValue::Uncompressed(c) => {
                plaintext_log_upload(c, self.sink_url.clone());
            }
        }
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
            let log_string = (self.format)(&record);

            // First, write whole formatted string
            encoder.borrow_mut().add_line(log_string);

            // Rotate the buffer based on a threshold
            let current_size = {
                let encoder = encoder.borrow();
                encoder.len()
            };
            if current_size < self.threshold {
                println!("Buffer {} of {}", current_size, self.threshold);
                // Compressed log didn't hit the size threshold
                return;
            }
            // Save the memory buffer using already acquired encoder instance.
            // With this approach it wont require us to manually drop a lock on encodr,
            // just to acquire it again inside rotate() function.
            let data = self.rotate(&encoder).expect("Unable to rotate the buffer");

            match data {
                FinishValue::Compressed(c) => {
                    debug_eprintln!("Sending {} bytes", c.compressed_plaintext_logs.len());
                    compressed_log_upload(c, self.sink_url.clone());
                }
                FinishValue::Uncompressed(c) => {
                    debug_eprintln!("Sending {} lines", c.logs.len());
                    plaintext_log_upload(c, self.sink_url.clone());
                }
            }
        }
    }

    fn flush(&self) {}
}

// #[test]
// fn logger() {
//     use actix::spawn;
//     use actix::Actor;
//     use actix::System;
//     use chrono::Local;
//     use flate2::read::ZlibDecoder;
//     use futures::future::IntoFuture;
//     use futures::future::{lazy, ok};
//     use std::any::Any;
//     use std::io;
//     use std::sync::mpsc;
//     let (tx, rx) = mpsc::channel();

//     let addr = LogClient::mock(Box::new(move |v, _ctx| -> Box<dyn Any> {
//         if let Some(msg) = v.downcast_ref::<LogChunk>() {
//             println!("Msg {:?}", msg.0);
//             tx.send(msg.0.clone()).unwrap();
//             System::current().stop();
//         } else {

//         }
//         Box::new(Some(()))
//     }));

//     let system = System::new("test");

//     let format = Box::new(|record: &Record| {
//         let timestamp = Local::now();
//         format!(
//             "{} {:<5} [{}] {}\n",
//             timestamp.format("%Y-%m-%d %H:%M:%S"),
//             record.level().to_string(),
//             record.module_path().unwrap_or_default(),
//             record.args()
//         )
//     });
//     let logger =
//         Logger::with_level(Level::Trace, Compression::Fast, 128, addr.start(), format).unwrap();

//     spawn(lazy(|| {
//         log::set_boxed_logger(Box::new(logger)).expect("Unable to set boxed logger");
//         log::set_max_level(Level::Trace.to_level_filter());
//         println!("foo");
//         info!("This log line is very long and it uses placeholders to verify that they are properly filled {} {:?}", "Hello, world!", 123_456_789u64);
//         ok(())
//     }).into_future());
//     system.run();

//     let data = rx.recv();
//     let data = data.as_ref().unwrap();
//     let mut decoder = ZlibDecoder::new(&data[..]);
//     let mut output: Vec<u8> = Vec::new();
//     io::copy(&mut decoder, &mut output).expect("Unable to copy data from decoder to output buffer");
//     let s = String::from_utf8(output).unwrap();
//     assert!(s.ends_with("This log line is very long and it uses placeholders to verify that they are properly filled Hello, world! 123456789\n"), "{}", s);
// }
