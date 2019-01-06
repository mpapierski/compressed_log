#![warn(clippy::all)]
#![allow(clippy::pedantic)]

pub mod builder;
pub mod client;
pub mod format;
pub mod logger;
pub mod lz4;
#[macro_use]
extern crate actix;
#[macro_use]
extern crate failure;

#[macro_use]
#[cfg(test)]

extern crate log;
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
