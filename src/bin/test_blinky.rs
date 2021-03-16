#![cfg_attr(test, allow(unused_attributes))]
#![cfg_attr(all(not(test), target_arch = "arm"), no_std)]
#![cfg_attr(target_arch = "arm", no_main)]

#[allow(unused_imports)]
use cortex_m_rt::{entry, exception};
pub const HEAP_SIZE_BYTES: usize = 8192;
use loadstone_lib as _;

#[cfg(target_arch = "arm")]
#[entry]
fn main() -> ! {
    use blue_hal::{drivers::efm32gg11b::gpio::Gpio, efm32pac};
    use blue_hal::hal::gpio::OutputPin;

    let mut peripherals = efm32pac::Peripherals::take().unwrap();
    let gpio = Gpio::new(peripherals.GPIO, &mut peripherals.CMU);

    let mut led_a = gpio.pa4.as_output();
    let mut led_b = gpio.pa5.as_output();
    led_a.set_low();
    led_b.set_low();
    panic!("Testing semihosting");
}

#[cfg(not(target_arch = "arm"))]
fn main() {}
