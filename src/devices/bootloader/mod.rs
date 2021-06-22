//! Generic Bootloader.
//!
//! This module contains all bootloader functionality, with
//! the exception of how to construct one. Construction is
//! handled by the `port` module as it depends on board
//! specific information.
use super::{
    boot_metrics::{boot_metrics_mut, BootMetrics, BootPath},
    image::{self, Bank, Image},
    traits::{Flash, Serial},
};
use crate::error::Error;
use blue_hal::{
    duprintln,
    hal::{flash, time},
    KB,
};
use core::{cmp::min, marker::PhantomData, mem::size_of};
use cortex_m::peripheral::SCB;
use defmt::{info, warn};
use nb::block;
use ufmt::uwriteln;
use crate::devices::update_signal::ReadUpdateSignal;

/// Operations related to copying images between flash chips.
mod copy;
/// Operations related to serial recovery when there's no fallback to restore to.
mod recover;
/// Operations related to restoring an image when there's no current one to boot.
mod restore;
/// Operations related to updating images with newer ones.
mod update;

/// Main bootloader struct.
// Members are public for the `ports` layer to be able to construct them freely and easily.
pub struct Bootloader<EXTF: Flash, MCUF: Flash, SRL: Serial, T: time::Now, R: image::Reader, RUS: ReadUpdateSignal> {
    pub(crate) mcu_flash: MCUF,
    pub(crate) external_banks: &'static [image::Bank<<EXTF as flash::ReadWrite>::Address>],
    pub(crate) mcu_banks: &'static [image::Bank<<MCUF as flash::ReadWrite>::Address>],
    pub(crate) external_flash: Option<EXTF>,
    pub(crate) serial: Option<SRL>,
    pub(crate) boot_metrics: BootMetrics,
    pub(crate) start_time: Option<T::I>,
    pub(crate) recovery_enabled: bool,
    pub(crate) update_signal: Option<RUS>,
    pub(crate) greeting: &'static str,
    pub(crate) _marker: PhantomData<R>,
}

impl<EXTF: Flash, MCUF: Flash, SRL: Serial, T: time::Now, R: image::Reader, RUS: ReadUpdateSignal>
    Bootloader<EXTF, MCUF, SRL, T, R, RUS>
{
    /// Main bootloader routine.
    ///
    /// In case the MCU flash's main bank contains a valid image, an update is attempted.
    /// (Any valid image with a different signature in the top occupied external bank is
    /// considered "newer" for the purposes of updating). The golden image, if available,
    /// is *never* considered newer than the current MCU image, as it exists only as a final
    /// resort fallback.
    ///
    /// After attempting or skipping the update process, the bootloader attempts to boot
    /// the current MCU image. In case of failure, the following steps are attempted:
    ///
    /// * Verify each bank in ascending order. If any is found to contain a valid
    /// image, copy it to bootable MCU flash bank and attempt to boot it.
    /// * Verify golden image. If valid, copy to bootable MCU flash bank and attempt to boot.
    /// * If golden image not available or invalid, proceed to recovery mode.
    pub fn run(mut self) -> ! {
        self.verify_bank_correctness();
        duprintln!(self.serial, "");
        duprintln!(self.serial, "{}", self.greeting);
        if let Some(image) = self.latest_bootable_image() {
            duprintln!(self.serial, "Attempting to boot from default bank.");
            match self.boot(image).unwrap_err() {
                Error::BankInvalid => {
                    info!("Attempted to boot from invalid bank. Restoring image...")
                }
                Error::BankEmpty => {
                    info!("Attempted to boot from empty bank. Restoring image...")
                }
                Error::SignatureInvalid => {
                    info!("Signature invalid for stored image. Restoring image...")
                }
                _ => info!("Unexpected boot error. Restoring image..."),
            };
        }

        match self.restore() {
            Ok(image) => self.boot(image).expect("FATAL: Failed to boot from verified image!"),
            Err(e) => {
                info!("Failed to restore. Error: {:?}", e);

                if self.recovery_enabled {
                    self.recover();
                } else {
                    panic!("FATAL: Failed to boot, and serial recovery is not supported.");
                }
            }
        }
    }
    /// Makes several sanity checks on the flash bank configuration.
    pub fn verify_bank_correctness(&self) {
        // There is at most one golden bank between internal and external flash
        let total_golden = self.external_banks.iter().filter(|b| b.is_golden).count()
            + self.mcu_banks.iter().filter(|b| b.is_golden).count();
        assert!(total_golden <= 1);

        // There is only one bootable MCU bank
        assert_eq!(self.mcu_banks().filter(|b| b.bootable).count(), 1);

        // Banks are sequential across flash chips
        let all_bank_indices =
            self.mcu_banks().map(|b| b.index).chain(self.external_banks().map(|b| b.index));
        all_bank_indices.fold(0, |previous, current| {
            assert!(previous + 1 == current, "Flash banks are not in sequence!");
            current
        });

        // Either there's external flash, or there's no external flash and no banks.
        assert!(
            self.external_flash.is_some() ||
            (self.external_flash.is_none() && self.external_banks().count() == 0),
            "Incorrect external flash configuration"
        );
    }

    /// Boots into a given memory bank.
    pub fn boot(&mut self, image: Image<MCUF::Address>) -> Result<!, Error> {
        warn!("Jumping to a new firmware image. This will break `defmt`.");
        let image_location_raw: usize = image.location().into();
        let time_ms = self.start_time.and_then(|t| Some((T::now() - t).0));
        self.boot_metrics.boot_time_ms = time_ms;

        // NOTE(Safety): Thoroughly unsafe operations, for obvious reasons: We are jumping to an
        // entirely different firmware image! We have to assume everything is at the right place,
        // or literally anything could happen here. No turning back after entering this unsafe block.
        unsafe {
            let initial_stack_pointer = *(image_location_raw as *const u32);
            let reset_handler_pointer =
                *((image_location_raw + size_of::<u32>()) as *const u32) as *const ();
            let reset_handler = core::mem::transmute::<*const (), fn() -> !>(reset_handler_pointer);
            (*SCB::ptr()).vtor.write(image_location_raw as u32);
            *boot_metrics_mut() = self.boot_metrics.clone();
            #[allow(deprecated)]
            cortex_m::register::msp::write(initial_stack_pointer);
            reset_handler()
        }
    }

    pub fn boot_bank(&self) -> image::Bank<MCUF::Address> {
        self.mcu_banks().find(|b| b.bootable).unwrap()
    }

    /// Returns an iterator of all MCU flash banks.
    pub fn mcu_banks(&self) -> impl Iterator<Item = image::Bank<MCUF::Address>> {
        self.mcu_banks.iter().cloned()
    }

    /// Returns an iterator of all external flash banks.
    pub fn external_banks(&self) -> impl Iterator<Item = image::Bank<EXTF::Address>> {
        self.external_banks.iter().cloned()
    }
}

#[cfg(test)]
#[doc(hidden)]
pub mod doubles {
    use blue_hal::{
        hal::{
            doubles::{
                error::FakeError,
                flash::{Address, FakeFlash},
                serial::SerialStub,
                time::MockSysTick,
            },
            null::NullFlash,
        },
        utilities::memory::doubles::FakeAddress,
    };
    use crate::devices::update_signal::{ReadUpdateSignal, UpdatePlan};

    pub struct FakeReader;

    impl Reader for FakeReader {
        fn image_at<A, F>(_flash: &mut F, _bank: Bank<A>) -> Result<Image<A>, error::Error>
        where
            A: blue_hal::utilities::memory::Address,
            F: blue_hal::hal::flash::ReadWrite<Address = A>,
            error::Error: From<F::Error>,
        {
            unimplemented!()
        }
    }

    pub struct FakeUpdateSignal;
    impl ReadUpdateSignal for FakeUpdateSignal {
        fn read_update_plan(&self) -> UpdatePlan { UpdatePlan::Any }
    }

    pub type BootloaderDouble =
        super::Bootloader<FakeFlash, FakeFlash, SerialStub, MockSysTick, FakeReader, FakeUpdateSignal>;

    impl BootloaderDouble {
        pub fn new() -> Self {
            BootloaderDouble {
                mcu_flash: FakeFlash::new(Address(0)),
                external_banks: &[],
                mcu_banks: &[],
                external_flash: Some(FakeFlash::new(Address(0))),
                serial: Some(SerialStub),
                boot_metrics: BootMetrics::default(),
                start_time: None,
                recovery_enabled: false,
                greeting: "I'm a fake bootloader!",
                _marker: Default::default(),
                update_signal: None,
            }
        }

        pub fn with_mcu_banks(self, mcu_banks: &'static [Bank<Address>]) -> Self {
            Self { mcu_banks, ..self }
        }

        pub fn with_external_banks(self, external_banks: &'static [Bank<Address>]) -> Self {
            Self { external_banks, ..self }
        }
    }

    use crate::{
        devices::{
            boot_metrics::BootMetrics,
            image::{Bank, Image, Reader},
        },
        error,
    };
    impl error::Convertible for FakeError {
        fn into(self) -> error::Error {
            error::Error::DeviceError("Something fake happened (test error)")
        }
    }
}
