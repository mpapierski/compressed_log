#[allow(unused_macros)]
macro_rules! debug_eprintln {
    () => ({
        #[cfg(debug_assertions)]
        eprint!("\n")
    });
    ($($arg:tt)*) => ({
        #[cfg(debug_assertions)]
        eprintln!($($arg)*);
    })
}

#[test]
fn test_debug_print() {
    debug_eprintln!();
    debug_eprintln!("Hello, world!");
}
