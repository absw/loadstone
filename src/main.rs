#![cfg_attr(test, allow(unused_imports))]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(not(test))]
extern crate panic_semihosting; // logs messages to the host stderr

use cortex_m_rt::entry;
use stm32f4::stm32f429;
use cortex_m_semihosting::hprintln;

#[cfg(not(test))]
#[entry]
fn main() -> ! {
    let peripherals = stm32f429::Peripherals::take().unwrap();

    peripherals.RCC.ahb1enr.write(|w| w.gpioben().bit(true));
    peripherals.GPIOB.moder.write(|w| w.moder7().bits(0b01));

    loop {
        cortex_m::asm::delay(2_000_000);
        hprintln!("Turning LED on").unwrap();
        peripherals.GPIOB.bsrr.write(|w| w.bs7().bit(true));
        cortex_m::asm::delay(2_000_000);
        hprintln!("Turning LED off").unwrap();
        peripherals.GPIOB.bsrr.write(|w| w.br7().bit(true));
    }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn foo() {
    println!("tests work!");
    assert!(2 == 3);
  }
}
