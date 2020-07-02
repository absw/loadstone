//! Driver implementations for all supported platforms. They offer
//! a safe API, and are
//! [typestate](https://rust-embedded.github.io/book/static-guarantees/typestate-programming.html)
//! based whenever possible.

#[macro_use]
pub mod gpio;
pub mod rcc;
#[macro_use]
pub mod serial;
pub mod qspi;
pub mod spi;
