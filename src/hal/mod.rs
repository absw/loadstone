//! Hardware Abstraction Layer, containing interfaces
//! for low level drivers.
#![macro_use]

pub mod gpio;
pub mod serial;
pub mod qspi;
pub mod spi;
pub mod time;
pub mod led;
pub mod flash;

#[cfg(not(target_arch = "arm"))]
#[doc(hidden)]
pub mod doubles;
