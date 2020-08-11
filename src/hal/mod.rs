//! Hardware Abstraction Layer, containing interfaces
//! for low level drivers.
#![macro_use]

pub mod flash;
pub mod gpio;
pub mod led;
pub mod qspi;
pub mod serial;
pub mod spi;
pub mod time;

#[cfg(not(target_arch = "arm"))]
#[doc(hidden)]
pub mod doubles;
