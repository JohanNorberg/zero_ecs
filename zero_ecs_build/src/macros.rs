#[allow(unused_macros, unused_imports)]
pub use quote::format_ident;
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        println!("cargo:warning={}", format_args!($($arg)*));
    };
}
#[macro_export]
macro_rules! fident {
    ($name:expr) => {
        format_ident!("{}", $name)
    };
}
