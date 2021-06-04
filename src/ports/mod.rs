//! Full project ports for specific targets. They mainly
//! provide a method to construct a generic bootloader from
//! specific parts.

#[allow(unused)]
use blue_hal::port;

#[cfg(feature = "stm32f412")]
port!(stm32f412: [bootloader, boot_manager,]);

#[cfg(feature = "wgm160p")]
port!(wgm160p: [bootloader,]);
