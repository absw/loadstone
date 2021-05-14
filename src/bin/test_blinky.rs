#![cfg_attr(test, allow(unused_attributes))]
#![cfg_attr(all(not(test), target_arch = "arm"), no_std)]
#![cfg_attr(target_arch = "arm", no_main)]

#[allow(unused_imports)]
use cortex_m_rt::{entry, exception};
use loadstone_lib as _;

#[cfg(target_arch = "arm")]
#[entry]
fn main() -> ! {
    use blue_hal::{
        drivers::efm32gg11b::{
            flash::{self, Flash},
            gpio::Gpio,
        },
        efm32pac,
        hal::{
            flash::ReadWrite,
            gpio::{InputPin, OutputPin},
        },
        KB,
    };

    let mut peripherals = efm32pac::Peripherals::take().unwrap();
    let gpio = Gpio::new(peripherals.GPIO, &mut peripherals.CMU);
    let mut flash = Flash::new(peripherals.MSC);

    let mut led_a = gpio.pa4.as_output();
    let mut led_b = gpio.pa5.as_output();
    let button_a = gpio.pd6.as_input();
    let button_b = gpio.pd8.as_input();
    use cortex_m_semihosting::hprintln;

    hprintln!("Hello from Blinky!").unwrap();

    hprintln!("Testing flash chip read/write cycle").unwrap();
    let safe_location = flash::Address(0x00005000);
    let test_buffer = [0xAA; KB!(10)];
    let mut expected_buffer = [0u8; KB!(10)];

    nb::block!(flash.write(safe_location, &test_buffer)).unwrap();
    nb::block!(flash.read(safe_location, &mut expected_buffer)).unwrap();
    assert_eq!(test_buffer, expected_buffer);

    hprintln!("Press buttons to trigger LEDs (testing GPIO)").unwrap();
    loop {
        if button_a.is_high() {
            led_a.set_low();
        } else {
            led_a.set_high();
        }
        if button_b.is_high() {
            led_b.set_low();
        } else {
            led_b.set_high();
        }
    }
}

#[cfg(not(target_arch = "arm"))]
fn main() {}
