//! # Secure Bootloader Library
//!
//! This crate contains all functionality for the
//! secure bootloader project in library form.
#![feature(never_type)]
#![feature(bool_to_option)]
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
extern crate panic_abort;

#[macro_use]
extern crate derive_is_enum_variant;
extern crate static_assertions;

#[macro_use]
pub mod utilities {
    pub mod bitwise;
    #[macro_use]
    pub mod macros;
}

/// Hardware Abstraction Layer, containing interfaces
/// for low level drivers.
#[macro_use]
pub mod hal;
pub mod devices;
#[macro_use]
pub mod drivers;
pub mod ports;
pub mod error;
pub mod pin_configuration;
