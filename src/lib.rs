#![warn(clippy::all)]
#![allow(clippy::pedantic)]
#![forbid(unsafe_code)]

#[macro_use]
mod macros;

pub mod builder;
pub mod client;
pub mod compression;
pub mod formatter;
pub mod logger;
#[macro_use]
extern crate serde_derive;
