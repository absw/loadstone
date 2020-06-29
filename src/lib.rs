//! # Secure Bootloader Library
//!
//! This crate contains all functionality for the
//! secure bootloader project in library form.
#![feature(never_type)]
#![cfg_attr(test, allow(unused_imports))]
#![cfg_attr(not(test), no_std)]

#[cfg(feature = "stm32f407")]
#[doc(hidden)]
pub use stm32f4::stm32f407 as stm32pac;
#[cfg(feature = "stm32f412")]
#[doc(hidden)]
pub use stm32f4::stm32f412 as stm32pac;
#[cfg(feature = "stm32f429")]
#[doc(hidden)]
pub use stm32f4::stm32f429 as stm32pac;
#[cfg(feature = "stm32f469")]
#[doc(hidden)]
pub use stm32f4::stm32f469 as stm32pac;

#[cfg(target_arch = "arm")]
extern crate panic_semihosting; // logs messages to the host stderr

#[macro_use]
extern crate derive_is_enum_variant;

#[macro_use]
pub mod drivers;

/// Hardware Abstraction Layer, containing interfaces
/// for low level drivers.
#[macro_use]
pub mod hal;
pub mod devices;
pub mod pin_configuration;
