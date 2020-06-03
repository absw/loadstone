#![cfg_attr(test, allow(unused_attributes))]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

#[allow(unused_imports)]
use cortex_m_rt::entry;

#[cfg(not(test))]
#[entry]
fn main() -> ! {
    use secure_bootloader_lib::{
        self,
        drivers::{gpio::GpioExt, rcc::RccExt, serial, serial::UsartExt},
        hal,
        hal::{gpio::OutputPin, serial::Write, time::Bps},
        uprint, uprintln,
        stm32pac
    };

    let mut peripherals = stm32pac::Peripherals::take().unwrap();

    #[cfg(feature = "stm32f407")]
    let gpioa = peripherals.GPIOA.split(&mut peripherals.RCC);
    #[cfg(any(feature = "stm32f429", feature = "stm32f469"))]
    let gpiob = peripherals.GPIOB.split(&mut peripherals.RCC);
    #[cfg(any(feature = "stm32f429", feature = "stm32f469", feature = "stm32f407"))]
    let gpiod = peripherals.GPIOD.split(&mut peripherals.RCC);

    let clocks = peripherals
        .RCC
        .constrain()
        .sysclk(hal::time::MegaHertz(180))
        .hclk(hal::time::MegaHertz(84))
        .pclk1(hal::time::MegaHertz(42))
        .pclk2(hal::time::MegaHertz(84))
        .require_pll48clk()
        .freeze();

    // Bring up serial communications (pins are MCU specific)
    #[cfg(feature = "stm32f429")]
    let (serial, tx, rx) = (peripherals.USART2, gpiod.pd5, gpiod.pd6);
    #[cfg(feature = "stm32f469")]
    let (serial, tx, rx) = (peripherals.USART3, gpiob.pb10, gpiob.pb11);
    #[cfg(feature = "stm32f407")]
    let (serial, tx, rx) = (peripherals.USART2, gpioa.pa2, gpioa.pa3);

    let serial_config = serial::config::Config::default().baudrate(Bps(115_200));
    let mut serial = serial.wrap((tx,rx), serial_config, clocks).unwrap();

    // Bring up blinky
    #[cfg(feature = "stm32f429")]
    let mut led_pin = gpiob.pb7;
    #[cfg(feature = "stm32f469")]
    let mut led_pin = gpiod.pd4;
    #[cfg(feature = "stm32f407")]
    let mut led_pin = gpiod.pd14;

    loop {
        cortex_m::asm::delay(20_000_000);
        led_pin.set_high();
        uprintln!(serial, "I switched the led off!");
        cortex_m::asm::delay(20_000_000);
        uprintln!(serial, "I switched the led on!");
        led_pin.set_low();
    }
}
