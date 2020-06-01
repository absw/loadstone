#![feature(never_type)]
#![cfg_attr(test, allow(unused_imports))]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(not(test))]
extern crate panic_semihosting; // logs messages to the host stderr

#[macro_use]
pub mod drivers;
#[macro_use]
pub mod hal;
pub mod pin_configuration;

use crate::{
    drivers::{gpio::GpioExt, rcc::RccExt, serial},
    hal::{gpio::OutputPin, serial::Write},
};
use cortex_m_rt::entry;
use stm32f4::stm32f429;

#[cfg(not(test))]
#[entry]
fn main() -> ! {
    let mut peripherals = stm32f429::Peripherals::take().unwrap();
    let gpiob = peripherals.GPIOB.split(&mut peripherals.RCC);
    let gpiod = peripherals.GPIOD.split(&mut peripherals.RCC);

    let clock_configuration = peripherals
        .RCC
        .constrain()
        .cfgr
        .sysclk(hal::time::MegaHertz(180))
        .hclk(hal::time::MegaHertz(84))
        .pclk1(hal::time::MegaHertz(42))
        .pclk2(hal::time::MegaHertz(84))
        .require_pll48clk();

    let clocks = clock_configuration.freeze();

    let mut serial = serial::Serial::usart3(
        peripherals.USART3,
        (gpiod.pd8, gpiod.pd9),
        serial::config::Config::default().baudrate(hal::time::Bps(115_200)),
        clocks,
    )
    .unwrap();

    let mut led_pin = gpiob.pb7;
    loop {
        cortex_m::asm::delay(20_000_000);
        led_pin.set_high();
        uprintln!(serial, "I switched the led off!");
        cortex_m::asm::delay(20_000_000);
        uprintln!(serial, "I switched the led on!");
        led_pin.set_low();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn foo() {
        println!("tests work!");
        assert!(3 == 3);
    }
}
