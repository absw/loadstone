#![cfg_attr(test, allow(unused_attributes))]
#![cfg_attr(all(not(test), target_arch = "arm"), no_std)]
#![cfg_attr(target_arch = "arm", no_main)]

#[allow(unused_imports)]
use cortex_m_rt::{entry, exception};
use secure_bootloader_lib::{
    drivers::{
        led,
        stm32f4::{gpio::GpioExt, rcc::Clocks, systick},
    },
    hal::{led::Toggle, time::Milliseconds},
    stm32pac,
};

#[cfg(target_arch = "arm")]
#[entry]
fn main() -> ! {
    let mut peripherals = stm32pac::Peripherals::take().unwrap();
    let gpioe = peripherals.GPIOE.split(&mut peripherals.RCC);
    let clocks = Clocks::hardcoded(&peripherals.FLASH, peripherals.RCC);
    let mut led = led::MonochromeLed::new(gpioe.pe1, led::LogicLevel::Inverted);

    loop {
        cortex_m::asm::delay(5_000_000);
        led.toggle();
    }
}

#[cfg(not(target_arch = "arm"))]
fn main() {}
