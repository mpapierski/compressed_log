#[macro_use]
extern crate log;
use compressed_log::builder::LoggerBuilder;
use compressed_log::compression::Compression;
use log::{Level, Record};

fn main() {
    // Initialize compressed logger
    let level = Level::Trace;
    let logger = LoggerBuilder::default()
        .set_level(level)
        .set_compression_level(Compression::Slow)
        .set_sink_url("http://127.0.0.1:9999/sink/")
        .set_threshold(1024)
        .set_format(Box::new(|record: &Record| format!("{}\n", record.args())))
        .build()
        .unwrap();
    log::set_boxed_logger(Box::new(logger)).unwrap();
    log::set_max_level(level.to_level_filter());
    eprintln!("Start");

    loop {
        error!("Look at me logging!");
    }
}
