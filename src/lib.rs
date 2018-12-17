pub mod builder;
pub mod format;
pub mod logger;
pub mod lz4;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
