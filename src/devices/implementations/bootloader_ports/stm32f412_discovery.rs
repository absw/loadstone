use crate::{
    devices::{
        implementations::{
            flash::micron_n25q128a::{self, MicronN25q128a},
            led::{LogicLevel, MonochromeLed},
        },
        interfaces::{
            flash::{Read, Write},
            led::Toggle,
        },
    },
    drivers::{
        gpio::{GpioExt, *},
        qspi::{self, mode, QuadSpi},
        rcc::Clocks,
        serial::{self, UsartExt},
    },
    hal::{self, serial::Write as SerialWrite},
    pin_configuration::*,
    stm32pac::{Peripherals, USART6},
};

use crate::stm32pac::QUADSPI;
use cortex_m::asm::delay;

// Flash pins and typedefs
type QspiPins = (Pb2<AF9>, Pg6<AF10>, Pf8<AF10>, Pf9<AF10>, Pf7<AF9>, Pf6<AF9>);
type Qspi = QuadSpi<QspiPins, mode::Single>;
type Flash = MicronN25q128a<Qspi>;

// Serial pins and typedefs
type UsartPins = (Pg14<AF8>, Pg9<AF8>);
type Serial = serial::Serial<USART6, UsartPins>;

/// Top level Bootloader type for the stm32f412 Discovery board
pub struct Bootloader {
    _flash: Flash,
    _serial: Serial,
}

impl Bootloader {
    fn build_and_test_flash(serial: &mut Serial, pins: QspiPins, qspi: QUADSPI) -> Flash {
        let qspi_config = qspi::Config::<mode::Single>::default().with_flash_size(24).unwrap();
        let qspi = Qspi::from_config(qspi, pins, qspi_config).unwrap();

        let mut flash = Flash::new(qspi).unwrap_or_else(|_| {
            uprintln!(serial, "* Flash manufacturer ID read failed!");
            panic!()
        });

        // Read, increase, write and read a magic number
        let mut magic_number_buffer = [0u8; 1];
        let mut new_magic_number_buffer = [0u8; 1];
        flash.read(micron_n25q128a::Address(0x0000_0000), &mut magic_number_buffer).unwrap();
        new_magic_number_buffer[0] = magic_number_buffer[0].wrapping_add(1);
        flash.write(micron_n25q128a::Address(0x0000_0000), &new_magic_number_buffer).unwrap();
        flash.read(micron_n25q128a::Address(0x0000_0000), &mut magic_number_buffer).unwrap();

        if magic_number_buffer != new_magic_number_buffer {
            uprintln!(serial, "* Flash read-write-read cycle failed!");
            panic!();
        }

        uprintln!(serial, "[POST]: Flash ID verification and RWR cycle passed");
        flash
    }

    pub fn new(mut peripherals: Peripherals) -> Bootloader {
        let gpiob = peripherals.GPIOB.split(&mut peripherals.RCC);
        let gpiog = peripherals.GPIOG.split(&mut peripherals.RCC);
        let gpiof = peripherals.GPIOF.split(&mut peripherals.RCC);
        let gpioe = peripherals.GPIOE.split(&mut peripherals.RCC);
        let mut post_led = MonochromeLed::new(gpioe.pe1, LogicLevel::Inverted);
        post_led.on();

        let clocks = Clocks::hardcoded(peripherals.FLASH, peripherals.RCC);
        let serial_config = serial::config::Config::default().baudrate(hal::time::Bps(115_200));
        let serial_pins = (gpiog.pg14, gpiog.pg9);
        let mut serial = peripherals.USART6.constrain(serial_pins, serial_config, clocks).unwrap();
        uprintln!(serial, "Initialising Secure Bootloader");

        let qspi_pins = (gpiob.pb2, gpiog.pg6, gpiof.pf8, gpiof.pf9, gpiof.pf7, gpiof.pf6);
        let flash = Self::build_and_test_flash(&mut serial, qspi_pins, peripherals.QUADSPI);

        post_led.off();
        Bootloader { _flash: flash, _serial: serial }
    }

    pub fn run(mut self) -> ! {
        loop {}
    }
}
