#![cfg_attr(test, allow(unused_attributes))]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

#[allow(unused_imports)]
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;

#[cfg(not(test))]
#[entry]
fn main() -> ! {
    use secure_bootloader_lib::{
        self,
        drivers::{gpio::GpioExt, rcc::RccExt, serial, serial::UsartExt},
        hal,
        hal::{gpio::OutputPin, serial::Write, time::Bps},
        stm32pac, uprint, uprintln,
    };

    let mut peripherals = stm32pac::Peripherals::take().unwrap();

    let clocks = peripherals
        .RCC
        .constrain()
        .sysclk(hal::time::MegaHertz(180))
        .hclk(hal::time::MegaHertz(84))
        .pclk1(hal::time::MegaHertz(42))
        .pclk2(hal::time::MegaHertz(84))
        .require_pll48clk()
        .freeze();

    loop {
        cortex_m::asm::delay(20_000_000);
        hprintln!("Hello World");
    }
}
