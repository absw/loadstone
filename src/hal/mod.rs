pub mod gpio;
#[macro_use]
pub mod serial;
pub mod spi;
pub mod qspi;
pub mod time;

#[cfg(not(target_arch = "arm"))]
#[doc(hidden)]
pub mod mock;
