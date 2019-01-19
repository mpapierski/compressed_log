#[macro_use]
extern crate log;
use actix::{Arbiter, System};
use compressed_log::builder::LoggerBuilder;
use compressed_log::lz4::Compression;
use failure::Error;
use futures::prelude::*;
use futures::sync::mpsc::channel;
use log::{Level, Record};
use std::io::{self, BufRead};
use std::thread;

fn stdin() -> impl Stream<Item = String, Error = io::Error> {
    let (mut tx, rx) = channel(1);
    thread::spawn(move || {
        let input = io::stdin();
        for line in input.lock().lines() {
            match tx.send(line).wait() {
                Ok(s) => tx = s,
                Err(_) => break,
            }
        }
    });
    rx.then(|e| e.unwrap())
}

fn main() -> Result<(), Error> {
    let sys = System::new("simple");
    // Initialize compressed logger
    let level = Level::Info;
    let logger = LoggerBuilder::default()
        .set_level(level)
        .set_compression_level(Compression::Slow)
        .set_sink_url("http://127.0.0.1:8000/sink/")
        .set_threshold(1024)
        .set_format(Box::new(|record: &Record| format!("{}\n", record.args())))
        .build()?;
    log::set_boxed_logger(Box::new(logger))?;
    log::set_max_level(level.to_level_filter());
    eprintln!("Start");

    Arbiter::spawn(
        stdin()
            .for_each(|line| {
                println!("Line: {:?}", line);
                info!("Line: {}", line);
                Ok(())
            })
            .map_err(|e| eprintln!("Stdin error: {}", e)),
    );

    sys.run();
    Ok(())
}
