//! Full project ports for specific targets. They mainly
//! provide a method to construct a generic bootloader from
//! specific parts.

use blue_hal::port;

#[cfg(feature = "stm32f412_discovery")]
port!(stm32f412_discovery: [bootloader, pin_configuration, boot_manager,]);

#[cfg(feature = "wgm160p")]
port!(wgm160p: [bootloader,]);
