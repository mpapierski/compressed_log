//! A logger that stores all log information in a compressed
//! buffer, and once the buffer is full it executes a rotation
//! strategy.

use crate::client::{LogChunk, LogClient};
use crate::lz4::{Compression, Encoder, EncoderBuilder, InMemoryEncoder};
use actix::{Addr, Arbiter};
use chrono::{DateTime, Local};
use failure::Error;
use futures::future::Future;
use log::{Level, Log, Metadata, Record};
use std::cell::RefCell;
use std::io::{self, Write};
use std::sync::Mutex;

/// A compressed logger structure.
pub struct Logger {
    level: Level,
    compression: Compression,
    threshold: usize,
    encoder: Mutex<RefCell<InMemoryEncoder>>,
    addr: Addr<LogClient>,
    format: Box<Fn(&Record) -> String + Sync + Send>,
}

impl Logger {
    pub fn new_encoder(compression: Compression) -> Result<Encoder<Vec<u8>>, Error> {
        // This is the empty buffer that needs to be passed as output of LZ4
        let buffer = Vec::<u8>::new();
        // TODO: This should be more configurable
        Ok(EncoderBuilder::new()
            .level(compression.to_level())
            .build(buffer)?)
    }

    // Create new Logger with given logging level
    pub fn with_level(
        level: Level,
        compression: Compression,
        threshold: usize,
        addr: Addr<LogClient>,
        format: Box<Fn(&Record) -> String + Sync + Send>,
    ) -> Result<Self, Error> {
        // Create new LZ4 encoder which may potentially fail.
        let encoder = Logger::new_encoder(compression)?;
        // Return the logger instance
        Ok(Self {
            level,
            compression,
            threshold,
            encoder: Mutex::new(RefCell::new(encoder)),
            addr,
            format,
        })
    }
    /// Rotates the internal LZ4 buffer and returns the compressed data
    /// buffer.
    fn rotate(&self, encoder: &RefCell<InMemoryEncoder>) -> Result<Vec<u8>, Error> {
        // Prepare new LZ4 encoder
        let new_encoder = Logger::new_encoder(self.compression)?;
        // Retrieve the old encoder by swapping it with the new one
        let old_encoder = encoder.replace(new_encoder);
        // Finish up the last bits of LZ4 stream and get the writer
        let (writer, result) = old_encoder.finish();
        // Check for any compression errors at the last step
        result?;
        // Return the data buffer
        Ok(writer)
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        // Unconditional rotation of log
        let encoder = self.encoder.lock().expect("Unable to acquire buffer lock");
        let data = self.rotate(&encoder).expect("Unable to rotate the buffer");
        if data.is_empty() {
            // No need to push empty data
            return;
        }
        // Send a chunk of data using the connection
        if let Err(_err) = self.addr.send(LogChunk(data)).wait() {
            debug_eprintln!("Unable to send log at the end of logger lifetime: {}", _err);
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
            let encoder = self.encoder.lock().expect("Unable to acquire buffer lock");
            // Serialize binary log record into the output buffer
            let log_string = (self.format)(&record);

            // First, write whole formatted string
            encoder
                .borrow_mut()
                .write_all(log_string.as_bytes())
                .expect("Unable to write data");

            // Flush internal buffers every time so we wouldn't accidentally rotate
            // without consistent data.
            encoder.borrow_mut().flush().expect("Unable to flush");

            // Rotate the buffer based on a threshold
            let current_size = {
                let encoder = encoder.borrow();
                let writer = encoder.writer();
                writer.len()
            };
            if current_size < self.threshold {
                // Compressed log didn't hit the size threshold
                return;
            }
            // Save the memory buffer using already acquired encoder instance.
            // With this approach it wouldn't require us to manually drop a lock on encodr,
            // just to acquire it again inside rotate() function.
            let data = self.rotate(&encoder).expect("Unable to rotate the buffer");
            // Acquire sender instance
            // Send a chunk of data using the connection
            debug_eprintln!("Sending {} bytes", data.len());
            Arbiter::spawn(self.addr.send(LogChunk(data)).map_err(|_e| {
                debug_eprintln!("Unable to send data {}", _e);
            }));
        }
    }

    fn flush(&self) {}
}

#[test]
fn logger() {
    use actix::Actor;
    use actix::System;
    use futures::future::IntoFuture;
    use futures::future::{lazy, ok};
    use lz4::Decoder;
    use std::any::Any;
    use std::io;
    use std::sync::mpsc;
    let (tx, rx) = mpsc::channel();

    let addr = LogClient::mock(Box::new(move |v, _ctx| -> Box<Any> {
        if let Some(msg) = v.downcast_ref::<LogChunk>() {
            println!("Msg {:?}", msg.0);
            tx.send(msg.0.clone()).unwrap();
            System::current().stop();
        } else {

        }
        Box::new(Some(()))
    }));

    let system = System::new("test");

    let format = Box::new(|record: &Record| {
        let timestamp = Local::now();
        format!(
            "{} {:<5} [{}] {}\n",
            timestamp.format("%Y-%m-%d %H:%M:%S"),
            record.level().to_string(),
            record.module_path().unwrap_or_default(),
            record.args()
        )
    });
    let logger =
        Logger::with_level(Level::Info, Compression::Fast, 128, addr.start(), format).unwrap();

    Arbiter::spawn(lazy(|| {
        log::set_boxed_logger(Box::new(logger)).expect("Unable to set boxed logger");
        log::set_max_level(Level::Info.to_level_filter());
        println!("foo");
        info!("This log line is very long and it uses placeholders to verify that they are properly filled {} {:?}", "Hello, world!", 123_456_789u64);
        ok(())
    }).into_future());
    system.run();

    let data = rx.recv().unwrap();
    let mut decoder = Decoder::new(&data[..]).expect("Unable to create decoder");
    let mut output: Vec<u8> = Vec::new();
    io::copy(&mut decoder, &mut output).expect("Unable to copy data from decoder to output buffer");
    let s = String::from_utf8(output).unwrap();
    assert!(s.ends_with("This log line is very long and it uses placeholders to verify that they are properly filled Hello, world! 123456789\n"), "{}", s);
}
