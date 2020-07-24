#![cfg_attr(test, allow(unused_attributes))]
#![cfg_attr(all(not(test), target_arch = "arm"), no_std)]
#![cfg_attr(target_arch = "arm", no_main)]

#[allow(unused_imports)]
use cortex_m_rt::{entry, exception};
use secure_bootloader_lib::drivers::stm32f4::rcc::Clocks;
use secure_bootloader_lib::stm32pac;
use secure_bootloader_lib::{hal::time::Seconds, drivers::{led, stm32f4::{systick, gpio::GpioExt}}};
use secure_bootloader_lib::hal::led::Toggle;

//#[cfg(target_arch = "arm")]
#[entry]
fn main() -> ! {
    let mut peripherals = stm32pac::Peripherals::take().unwrap();
    let cortex_peripherals = cortex_m::Peripherals::take().unwrap();

    let gpioe = peripherals.GPIOE.split(&mut peripherals.RCC);
    let clocks = Clocks::hardcoded(&peripherals.FLASH, peripherals.RCC);
    let systick = systick::SysTick::new(cortex_peripherals.SYST, clocks);
    let mut led = led::MonochromeLed::new(gpioe.pe1, led::LogicLevel::Inverted);

    loop {
        systick.wait(Seconds(1));
        led.toggle();
    }
}

#[cfg(not(target_arch = "arm"))]
fn main() {}
