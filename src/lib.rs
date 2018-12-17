pub mod builder;
pub mod client;
pub mod format;
pub mod logger;
pub mod lz4;
#[macro_use]
extern crate actix;
#[macro_use]
extern crate failure;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
