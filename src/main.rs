#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

#[allow(unused_imports)]
use cortex_m_rt::entry;

#[cfg(not(test))]
#[entry]
fn main() -> ! {
    use secure_bootloader_lib;

    use secure_bootloader_lib::{
        drivers::{gpio::GpioExt, rcc::RccExt, serial},
        hal,
        hal::{gpio::OutputPin, serial::Write},
        uprint, uprintln,
        stm32pac
    };

    let mut peripherals = stm32pac::Peripherals::take().unwrap();
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

    #[cfg(feature = "stm32f429")]
    let mut serial = serial::Serial::usart2(
        peripherals.USART2,
        (gpiod.pd5, gpiod.pd6),
        serial::config::Config::default().baudrate(hal::time::Bps(115_200)),
        clocks,
    )
    .unwrap();

    #[cfg(feature = "stm32f469")]
    let mut serial = serial::Serial::usart3(
        peripherals.USART3,
        (gpiob.pb10, gpiob.pb11),
        serial::config::Config::default().baudrate(hal::time::Bps(115_200)),
        clocks,
    )
    .unwrap();

    #[cfg(feature = "stm32f429")]
    let mut led_pin = gpiob.pb7;

    #[cfg(feature = "stm32f469")]
    let mut led_pin = gpiod.pd4;

    loop {
        cortex_m::asm::delay(20_000_000);
        led_pin.set_high();
        uprintln!(serial, "I switched the led off!");
        cortex_m::asm::delay(20_000_000);
        uprintln!(serial, "I switched the led on!");
        led_pin.set_low();
    }
}
