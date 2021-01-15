//! GPIO configuration and alternate functions for the [stm32f412 discovery](../../../../../../../../documentation/hardware/discovery.pdf).
use crate::devices::bootloader::Bootloader;
use crate::devices::image;
use crate::error::Error;
use core::mem::size_of;
use blue_hal::{drivers::{micron::n25q128a_flash::{self, MicronN25q128a}, stm32f4::{flash, qspi::{self, QuadSpi, mode}, rcc::Clocks, serial, systick::SysTick}}, hal::time, stm32pac};
use super::pin_configuration::*;

// Flash pins and typedefs
type QspiPins = (Pb2<AF9>, Pg6<AF10>, Pf8<AF10>, Pf9<AF10>, Pf7<AF9>, Pf6<AF9>);
type Qspi = QuadSpi<QspiPins, mode::Single>;
type ExternalFlash = MicronN25q128a<Qspi, SysTick>;

// Serial pins and typedefs
const EXTERNAL_NUMBER_OF_BANKS: usize = 2;
const EXTERNAL_BANK_MAX_IMAGE_SIZE: usize = {
    let (start, end) = (n25q128a_flash::MemoryMap::location(), n25q128a_flash::MemoryMap::end());
    let total_size = (end.0 - start.0) as usize;
    let size_without_header = total_size - size_of::<image::GlobalHeader>();
    let size_per_image = size_without_header / EXTERNAL_NUMBER_OF_BANKS;
    size_per_image - size_of::<image::ImageHeader>()
};

const MCU_NUMBER_OF_BANKS: usize = 1;
const MCU_BANK_MAX_IMAGE_SIZE: usize = {
    let (start, end) = (flash::MemoryMap::writable_start(), flash::MemoryMap::writable_end());
    let total_size = (end.0 - start.0) as usize;
    let size_without_header = total_size - size_of::<image::GlobalHeader>();
    let size_per_image = size_without_header / MCU_NUMBER_OF_BANKS;
    size_per_image - size_of::<image::ImageHeader>()
};

const fn min(a: usize, b: usize) -> usize { if a < b { a } else { b } }
const IMAGE_SIZE: usize = min(MCU_BANK_MAX_IMAGE_SIZE, EXTERNAL_BANK_MAX_IMAGE_SIZE);
const IMAGE_SIZE_WITH_HEADER: usize = IMAGE_SIZE + size_of::<image::ImageHeader>();

const fn external_image_offset(index: usize) -> n25q128a_flash::Address {
   n25q128a_flash::Address(n25q128a_flash::MemoryMap::location().0
        + (index * IMAGE_SIZE_WITH_HEADER) as u32)
}

const fn mcu_image_offset(index: usize) -> flash::Address {
    flash::Address(flash::MemoryMap::writable_start().0
        + (index * IMAGE_SIZE_WITH_HEADER) as u32)
}

static MCU_BANKS: [image::Bank<flash::Address>; MCU_NUMBER_OF_BANKS] = [
    image::Bank { index: 1, bootable: true, location: mcu_image_offset(0), size: IMAGE_SIZE, },
];

static EXTERNAL_BANKS: [image::Bank<n25q128a_flash::Address>; EXTERNAL_NUMBER_OF_BANKS] = [
    image::Bank { index: 2, bootable: false, location: external_image_offset(0), size: IMAGE_SIZE, },
    image::Bank { index: 3, bootable: false, location: external_image_offset(1), size: IMAGE_SIZE, },
];

impl Default for Bootloader<ExternalFlash, flash::McuFlash> {
    fn default() -> Self { Self::new() }
}

impl Bootloader<ExternalFlash, flash::McuFlash> {
    pub fn new() -> Self {
        let mut peripherals = stm32pac::Peripherals::take().unwrap();
        let cortex_peripherals = cortex_m::Peripherals::take().unwrap();
        let gpiob = peripherals.GPIOB.split(&mut peripherals.RCC);
        let gpiog = peripherals.GPIOG.split(&mut peripherals.RCC);
        let gpiof = peripherals.GPIOF.split(&mut peripherals.RCC);

        let mcu_flash = flash::McuFlash::new(peripherals.FLASH).unwrap();
        let clocks = Clocks::hardcoded(peripherals.RCC);

        SysTick::init(cortex_peripherals.SYST, clocks);
        SysTick::wait(time::Seconds(1)); // Gives time for the flash chip to stabilize after powerup

        //let serial_config = serial::config::Config::default().baudrate(time::Bps(115200));
        //let serial_pins = (gpiog.pg14, gpiog.pg9);
        //let mut serial = peripherals.USART6.constrain(serial_pins, serial_config, clocks).unwrap();
        //let cli = Cli::new(serial).unwrap();

        let qspi_pins = (gpiob.pb2, gpiog.pg6, gpiof.pf8, gpiof.pf9, gpiof.pf7, gpiof.pf6);
        let qspi_config = qspi::Config::<mode::Single>::default().with_flash_size(24).unwrap();
        let qspi = Qspi::from_config(peripherals.QUADSPI, qspi_pins, qspi_config).unwrap();
        let external_flash = ExternalFlash::with_timeout(qspi, time::Milliseconds(500)).unwrap();

        Bootloader { external_flash, mcu_flash, external_banks: &EXTERNAL_BANKS, mcu_banks: &MCU_BANKS }
    }
}

impl From<flash::Error> for Error {
    fn from(error: flash::Error) -> Self {
        match error {
            flash::Error::MemoryNotReachable => Error::DriverError("[MCU Flash] Memory not reachable"),
            flash::Error::MisalignedAccess => Error::DriverError("[MCU Flash] Misaligned memory access"),
        }
    }
}

impl From<n25q128a_flash::Error> for Error {
    fn from(error: n25q128a_flash::Error) -> Self {
        match error {
            n25q128a_flash::Error::TimeOut => Error::DriverError("[External Flash] Operation timed out"),
            n25q128a_flash::Error::QspiError => Error::DriverError("[External Flash] Qspi error"),
            n25q128a_flash::Error::WrongManufacturerId => Error::DriverError("[External Flash] Wrong manufacturer ID"),
            n25q128a_flash::Error::MisalignedAccess => Error::DriverError("[External Flash] Misaligned memory access"),
            n25q128a_flash::Error::AddressOutOfRange => Error::DriverError("[External Flash] Address out of range"),
        }
    }
}

impl From<serial::Error> for Error {
    fn from(error: serial::Error) -> Self {
        match error {
            serial::Error::Framing => Error::DriverError("[Serial] Framing error"),
            serial::Error::Noise => Error::DriverError("[Serial] Noise error"),
            serial::Error::Overrun => Error::DriverError("[Serial] Overrun error"),
            serial::Error::Parity => Error::DriverError("[Serial] Parity error"),
            serial::Error::Timeout => Error::DriverError("[Serial] Timeout error"),
            _ => Error::DriverError("[Serial] Unexpected serial error"),
        }
    }
}
