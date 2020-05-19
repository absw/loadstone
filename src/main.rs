#![cfg_attr(test, allow(unused_imports))]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(not(test))]
extern crate panic_semihosting; // logs messages to the host stderr

pub mod drivers;
pub mod hal;

use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use stm32f4::stm32f429;
use crate::drivers::gpio::GpioExt;
use crate::hal::gpio::OutputPin;


#[cfg(not(test))]
#[entry]
fn main() -> ! {
    let mut peripherals = stm32f429::Peripherals::take().unwrap();
    let mut gpiob = peripherals.GPIOB.split(&mut peripherals.RCC);
    let mut led_pin = gpiob.pb7.into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);

    loop {
        cortex_m::asm::delay(2_000_000);
        hprintln!("Turning LED on").unwrap();
        led_pin.set_high();
        cortex_m::asm::delay(2_000_000);
        hprintln!("Turning LED off").unwrap();
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
