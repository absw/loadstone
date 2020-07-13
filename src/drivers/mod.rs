//! Driver implementations for all supported platforms. They offer
//! a safe API, and are
//! [typestate](https://rust-embedded.github.io/book/static-guarantees/typestate-programming.html)
//! based whenever possible.

#[cfg(feature = "stm32f4_any")]
#[macro_use]
pub mod stm32f4 {
    #[macro_use]
    pub mod gpio;
    pub mod rcc;
    #[macro_use]
    pub mod serial;
    pub mod qspi;
    pub mod spi;
    pub mod systick;
    pub mod flash;
}

pub mod led;

#[cfg(feature = "stm32f412_discovery")]
mod micron {
    pub mod n25q128a_flash;
}

