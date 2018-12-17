#[macro_use]
extern crate log;
use compressed_log::logger::init;
use failure::Error;
use std::io::{self, BufRead};

fn main() -> Result<(), Error> {
    // Initialize compressed logger
    init()?;
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
