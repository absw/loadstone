//! Concrete bootloader construction and flash bank layout for the wgm160p

use blue_hal::{drivers::efm32gg11b::{clocks, flash::{self, Flash}}, efm32pac, hal::null::{NullFlash, NullSerial, NullSystick}};

use crate::{devices::{bootloader::Bootloader, image}, error::{self, Error}};
use blue_hal::KB;

const IMAGES_START: flash::Address = flash::Address(KB!(64));
const IMAGE_SIZE: usize = KB!(918);

const fn image_offset(index: usize) -> flash::Address {
    flash::Address(IMAGES_START.0 + (index * IMAGE_SIZE) as u32)
}
pub static MCU_BANKS: [image::Bank<flash::Address>; 2] = [
    image::Bank { index: 1, bootable: true, location: image_offset(0), size: IMAGE_SIZE, is_golden: false },
    image::Bank { index: 2, bootable: false, location: image_offset(1), size: IMAGE_SIZE, is_golden: false },
];
impl Bootloader<NullFlash, Flash, NullSerial, NullSystick> {
    pub fn new() -> Self {
        let mut peripherals = efm32pac::Peripherals::take().unwrap();
        let clocks = clocks::Clocks::new(peripherals.CMU, &mut peripherals.MSC);
        let mcu_flash = flash::Flash::new(peripherals.MSC, &clocks);

        #[cfg(feature = "serial")]
        compile_error!("Serial communications not yet supported in the wgm160p port");

        #[cfg(feature = "boot-time-metrics")]
        compile_error!("Boot time metrics not yet supported in the wgm160p port");

        Bootloader {
            mcu_flash,
            external_banks: &[],
            mcu_banks: &MCU_BANKS,
            external_flash: None,
            serial: None,
            boot_metrics: Default::default(),
            start_time: None,
        }
    }
}

impl error::Convertible for flash::Error {
    fn into(self) -> Error {
        match self {
            flash::Error::MemoryNotReachable => Error::DriverError("[MCU Flash] Memory not reachable"),
            flash::Error::MisalignedAccess => Error::DriverError("[MCU Flash] Misaligned memory access"),
            flash::Error::MemoryIsLocked => Error::DriverError("[MCU Flash] Memory is locked"),
            flash::Error::InvalidAddress => Error::DriverError("[MCU Flash] Address is invalid"),
        }
    }
}

impl error::Convertible for ! {
    fn into(self) -> Error { unimplemented!() }
}
