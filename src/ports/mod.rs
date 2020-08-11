//! Full project ports for specific targets. They mainly
//! provide a method to construct a generic bootloader from
//! specific parts.

#[cfg(feature = "stm32f412_discovery")]
port!(stm32f412_discovery: [bootloader, pin_configuration,]);
