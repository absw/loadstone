#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

use cortex_m_rt::entry;

#[cfg(not(any(test, doctest)))]
#[entry]
fn main() -> ! {
    use secure_bootloader_lib;
    use stm32f4::stm32f429;

    use secure_bootloader_lib::{
        drivers::{gpio::GpioExt, rcc::RccExt, serial},
        hal,
        hal::{gpio::OutputPin, serial::Write},
        uprint, uprintln,
    };

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

    let mut serial = serial::Serial::usart2(
        peripherals.USART2,
        (gpiod.pd5, gpiod.pd6),
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
