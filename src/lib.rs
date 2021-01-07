//! # Secure Bootloader Library
//!
//! This crate contains all functionality for the
//! secure bootloader project in library form.
#![feature(never_type)]
#![feature(bool_to_option)]
#![feature(array_value_iter)]
#![feature(associated_type_bounds)]
#![cfg_attr(test, allow(unused_imports))]
#![cfg_attr(target_arch = "arm", no_std)]

#[cfg(feature = "stm32f407")]
pub use stm32f4::stm32f407 as stm32pac;
#[cfg(feature = "stm32f412")]
pub use stm32f4::stm32f412 as stm32pac;
#[cfg(feature = "stm32f429")]
pub use stm32f4::stm32f429 as stm32pac;
#[cfg(feature = "stm32f469")]
pub use stm32f4::stm32f469 as stm32pac;

#[cfg(target_arch = "arm")]
use panic_abort as _;

#[cfg(target_arch = "arm")]
use defmt_rtt as _; // global logger

#[macro_use]
pub mod utilities {
    pub mod bitwise;
    pub mod iterator;
    pub mod memory;
    pub mod guard;
    pub mod buffer;
    pub mod xmodem;
    mod macros;
}

pub mod hal;
pub mod drivers;
pub mod devices;
pub mod error;
pub mod ports;
