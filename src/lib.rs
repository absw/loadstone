//! # Secure Bootloader Library
//!
//! This crate contains all functionality for the
//! secure bootloader project in library form.
#![feature(never_type)]
#![feature(bool_to_option)]
#![feature(array_value_iter)]
#![feature(associated_type_bounds)]
#![feature(alloc_error_handler)]
#![cfg_attr(test, allow(unused_imports))]
#![cfg_attr(target_arch = "arm", no_std)]

use alloc_cortex_m::CortexMHeap;
pub use blue_hal::stm32pac;

#[global_allocator]
pub static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

#[cfg(target_arch = "arm")]
#[alloc_error_handler]
fn oom(_: core::alloc::Layout) -> ! {
    loop {}
}

#[cfg(target_arch = "arm")]
use panic_abort as _;

#[cfg(target_arch = "arm")]
use defmt_rtt as _; // global logger

pub mod devices;
pub mod error;
pub mod ports;
