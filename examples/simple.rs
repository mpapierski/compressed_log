#[macro_use]
extern crate log;
use compressed_log::builder::LoggerBuilder;
use compressed_log::lz4::Compression;
use failure::Error;
use log::Level;
use std::io::{self, BufRead};

fn main() -> Result<(), Error> {
    // Initialize compressed logger
    let level = Level::Trace;
    let logger = LoggerBuilder::new()
        .set_level(level)
        .set_compression_level(Compression::Slow)
        .set_sink_url("http://127.0.0.1:8000/sink")
        .build()?;
    log::set_boxed_logger(Box::new(logger))?;
    log::set_max_level(level.to_level_filter());
    eprintln!("Start");

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        if line.starts_with("TRACE ") {
            trace!("{}", &line[6..]);
        } else if line.starts_with(" INFO ") {
            info!("{}", &line[6..]);
        } else if line.starts_with(" WARN ") {
            warn!("{}", &line[6..]);
        } else {
            panic!("Unknown prefix: {}", &line[0..6]);
        }
    }
    Ok(())
}
