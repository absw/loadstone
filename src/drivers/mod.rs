//! Driver implementations for all supported platforms. They offer
//! a safe API, and are
//! [typestate](https://rust-embedded.github.io/book/static-guarantees/typestate-programming.html)
//! based whenever possible.
#![macro_use]

/// Drivers for the stm32f4 family of microcontrollers.
#[cfg(feature = "stm32f4_any")]
#[macro_use]
pub mod stm32f4 {
    pub mod flash;
    pub mod gpio;
    #[cfg(feature = "stm32f412")]
    pub mod qspi;
    pub mod rcc;
    pub mod serial;
    pub mod spi;
    pub mod systick;
}

pub mod led;

/// Drivers for the Micron manufacturer (e.g. external flash).
#[cfg(feature = "stm32f412_discovery")]
pub mod micron {
    /// N25Q128A external flash chip
    pub mod n25q128a_flash;
}
