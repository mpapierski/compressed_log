macro_rules! debug_eprintln {
    () => ({
        #[cfg(any(debug_assertions,test,feature = "debug"))]
        eprint!("\n")
    });
    ($($arg:tt)*) => ({
        #[cfg(any(debug_assertions,test,feature = "debug"))]
        eprintln!($($arg)*);
    })
}

#[test]
fn test_debug_print() {
    debug_eprintln!();
    debug_eprintln!("Hello, world!");
}
