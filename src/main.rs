#![cfg_attr(test, allow(unused_attributes))]
#![cfg_attr(target_arch = "arm", no_std)]
#![cfg_attr(target_arch = "arm", no_main)]

#[allow(unused_imports)]
use cortex_m_rt::entry;

#[cfg(target_arch = "arm")]
#[entry]
fn main() -> ! {
    use cortex_m_semihosting::hprintln;
    use secure_bootloader_lib::{
        self, drivers::rcc::RccExt, hal, stm32pac,
    };
    let peripherals = stm32pac::Peripherals::take().unwrap();

    peripherals
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
        hprintln!("Hello World").unwrap();
    }
}

#[cfg(not(target_arch = "arm"))]
fn main() {}
