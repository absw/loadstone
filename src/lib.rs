#![feature(never_type)]
#![cfg_attr(test, allow(unused_imports))]
#![cfg_attr(not(test), no_std)]

#[cfg(feature = "stm32f429")]
pub use stm32f4::stm32f429 as stm32pac;

#[cfg(feature = "stm32f469")]
pub use stm32f4::stm32f469 as stm32pac;

#[cfg(not(test))]
extern crate panic_semihosting; // logs messages to the host stderr

#[macro_use]
pub mod drivers;
#[macro_use]
pub mod hal;

pub mod pin_configuration;
