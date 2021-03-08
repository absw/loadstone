//! Generic Bootloader.
//!
//! This module contains all bootloader functionality, with
//! the exception of how to construct one. Construction is
//! handled by the `port` module as it depends on board
//! specific information.
use super::{boot_metrics::{boot_metrics_mut, BootMetrics, BootPath}, image::{self, Bank, Image, GOLDEN_STRING, MAGIC_STRING}, traits::{Flash, Serial}};
use crate::{devices::cli::file_transfer::FileTransfer, error::Error};
use blue_hal::{
    duprintln,
    hal::{flash, serial, time},
    KB,
};
use core::{cmp::min, marker::PhantomData, mem::size_of};
use cortex_m::peripheral::SCB;
use defmt::{info, warn};
use ecdsa::{generic_array::typenum::Unsigned, SignatureSize};
use nb::block;
use p256::NistP256;
use ufmt::uwriteln;

pub struct Bootloader<EXTF: Flash, MCUF: Flash, SRL: Serial, T: time::Now>
{
    pub(crate) mcu_flash: MCUF,
    pub(crate) external_banks: &'static [image::Bank<<EXTF as flash::ReadWrite>::Address>],
    pub(crate) mcu_banks: &'static [image::Bank<<MCUF as flash::ReadWrite>::Address>],
    pub(crate) external_flash: Option<EXTF>,
    pub(crate) serial: SRL,
    pub(crate) boot_metrics: BootMetrics,
    pub(crate) start_time: T::I,
    pub(crate) _marker: PhantomData<T>,
}

const DEFAULT_BOOT_BANK: u8 = 1;

impl<EXTF: Flash, MCUF: Flash, SRL: Serial, T: time::Now> Bootloader<EXTF, MCUF, SRL, T>
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
    /// * Verify each external bank in ascending order. If any is found to contain a valid
    /// image, copy it to bootable MCU flash bank and attempt to boot it.
    /// * Verify golden image. If valid, copy to bootable MCU flash bank and attempt to boot.
    /// * If golden image not available or invalid, proceed to recovery mode.
    pub fn run(mut self) -> ! {
        let total_golden = self.external_banks.iter().filter(|b| b.is_golden).count()
            + self.mcu_banks.iter().filter(|b| b.is_golden).count();

        assert!(total_golden <= 1);
        duprintln!(self.serial, "-- Loadstone Initialised --");
        if let Some(image) = self.try_update_image() {
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
                self.recover();
            }
        }
    }

    // Attempts to update from any input flash
    fn try_update_image_from<F: Flash>(
        &mut self,
        current_image: Image<MCUF::Address>,
        mcu_flash: MCUF,
        mut source_flash: F,
        banks: impl Iterator<Item = Bank<F::Address>>,
    )
    {
        for bank in banks.filter(|b| !b.is_golden) {
            duprintln!(self.serial, "Scanning bank {:?} for a newer image...", bank.index);
            match image::image_at(&mut source_flash, bank) {
                Ok(image) if image.signature() != current_image.signature() => {
                    duprintln!(
                        self.serial,
                        "Replacing current image with bank {:?}...",
                        bank.index
                    );
                    unimplemented!();
                    //self.copy_image_from_external(*external_bank, *boot_bank, false).unwrap();
                    //self.boot_metrics.boot_path = BootPath::Updated { bank: external_bank.index };
                    //duprintln!(
                    //    self.serial,
                    //    "Replaced image with external bank {:?}.",
                    //    external_bank.index
                    //);
                    //return image::image_at(&mut self.mcu_flash, *boot_bank).ok();
                }
                Ok(_image) => break,
                _ => (),
            }
        }
    }

    /// If the current bootable (MCU flash) image is different from the top
    /// non-golden image, attempts to replace it. On failure, this process
    /// is repeated for all non-golden banks. Returns the current
    /// bootable image after the process, if available.
    fn try_update_image(&mut self) -> Option<Image<MCUF::Address>> {
        let boot_bank = self.mcu_banks.iter().find(|b| b.index == DEFAULT_BOOT_BANK).unwrap();
        duprintln!(self.serial, "Checking for image updates...");

        let current_image = if let Ok(image) = image::image_at(&mut self.mcu_flash, *boot_bank) {
            image
        } else {
            duprintln!(self.serial, "No current image.");
            return None;
        };

        if let Some(ref mut external_flash) = self.external_flash {
            for external_bank in self.external_banks.iter().filter(|b| !b.is_golden) {
                duprintln!(
                    self.serial,
                    "Scanning external bank {:?} for a newer image...",
                    external_bank.index
                );
                match image::image_at(external_flash, *external_bank) {
                    Ok(image) if image.signature() != current_image.signature() => {
                        duprintln!(
                            self.serial,
                            "Replacing current image with external bank {:?}...",
                            external_bank.index
                        );
                        self.copy_image_from_external(*external_bank, *boot_bank, false).unwrap();
                        self.boot_metrics.boot_path =
                            BootPath::Updated { bank: external_bank.index };
                        duprintln!(
                            self.serial,
                            "Replaced image with external bank {:?}.",
                            external_bank.index
                        );
                        return image::image_at(&mut self.mcu_flash, *boot_bank).ok();
                    }
                    Ok(_image) => break,
                    _ => (),
                }
            }
        }
        duprintln!(self.serial, "No newer image found.");
        Some(current_image)
    }

    /// Restores the first image available in the external banks, attempting to restore
    /// from the golden image as a last resort.
    fn restore(&mut self) -> Result<Image<MCUF::Address>, Error> {
        // Attempt to restore from normal image
        let output = self.mcu_banks.iter().find(|b| b.index == DEFAULT_BOOT_BANK).unwrap();
        for input_bank in self.external_banks.iter().filter(|b| !b.is_golden) {
            duprintln!(self.serial, "Attempting to restore from bank {:?}.", input_bank.index);
            if self.copy_image_from_external(*input_bank, *output, false).is_ok() {
                duprintln!(
                    self.serial,
                    "Restored image from external bank {:?}.",
                    input_bank.index
                );
                duprintln!(self.serial, "Verifying the image again in the boot bank...");
                self.boot_metrics.boot_path = BootPath::Restored { bank: input_bank.index };
                return Ok(image::image_at(&mut self.mcu_flash, *output)?);
            };
        }

        // Attempt to restore from golden image
        let golden_bank = self.external_banks.iter().find(|b| b.is_golden).unwrap();
        duprintln!(self.serial, "Attempting to restore from golden bank {:?}.", golden_bank.index);
        if self.copy_image_from_external(*golden_bank, *output, true).is_ok() {
            duprintln!(
                self.serial,
                "Restored image from external golden bank {:?}.",
                golden_bank.index
            );
            duprintln!(self.serial, "Verifying the image again in the boot bank...");
            self.boot_metrics.boot_path = BootPath::Restored { bank: golden_bank.index };
            return Ok(image::image_at(&mut self.mcu_flash, *output)?);
        };

        Err(Error::NoImageToRestoreFrom)
    }

    /// Enters recovery mode, which requests a golden image to be transferred via serial through
    /// the XMODEM protocol, then reboot.
    fn recover(&mut self) -> ! {
        duprintln!(self.serial, "-- Loadstone Recovery Mode --");
        duprintln!(self.serial, "Please send golden firmware image via XMODEM.");
        let golden_bank = self.external_banks.iter().find(|b| b.is_golden).unwrap();

        if let Some(ref mut external_flash) = self.external_flash {
            let blocks = self.serial.blocks(None);
            if external_flash.write_from_blocks(golden_bank.location, blocks).is_err() {
                duprintln!(
                    self.serial,
                    "FATAL: Failed to flash golden image during recovery mode."
                );
            }

            match image::image_at(external_flash, *golden_bank) {
                Ok(image) if !image.is_golden() => {
                    duprintln!(self.serial, "FATAL: Flashed image is not a golden image.")
                }
                Err(e) => {
                    duprintln!(self.serial, "FATAL: Image did not flash correctly.");
                    e.report(&mut self.serial);
                }
                _ => duprintln!(self.serial, "Finished flashing golden image."),
            }
        }

        duprintln!(self.serial, "Rebooting...");
        SCB::sys_reset();
    }

    /// Boots into a given memory bank.
    pub fn boot(&mut self, image: Image<MCUF::Address>) -> Result<!, Error> {
        warn!("Jumping to a new firmware image. This will break `defmt`.");
        let image_location_raw: usize = image.location().into();
        let time_ms = T::now() - self.start_time;
        self.boot_metrics.boot_time_ms = time_ms.0;

        // NOTE(Safety): Thoroughly unsafe operations, for obvious reasons: We are jumping to an
        // entirely different firmware image! We have to assume everything is at the right place,
        // or literally anything could happen here. After the interrupts are disabled, there is
        // no turning back.
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

    /// Returns an iterator of all MCU flash banks.
    pub fn mcu_banks(&self) -> impl Iterator<Item = image::Bank<MCUF::Address>> {
        self.mcu_banks.iter().cloned()
    }

    /// Returns an iterator of all external flash banks.
    pub fn external_banks(&self) -> impl Iterator<Item = image::Bank<EXTF::Address>> {
        self.external_banks.iter().cloned()
    }

    pub fn copy_image<I, O>()
        where
        I: flash::ReadWrite,
        Error: From<I::Error>,
        O: flash::ReadWrite,
        Error: From<O::Error>,
    {
    }

    /// Copy from external bank to MCU bank. This routine uses a significant amount
    /// of stack space to minimise flash erases and thus maximise flash usage efficiency.
    pub fn copy_image_from_external(
        &mut self,
        input_bank: image::Bank<EXTF::Address>,
        output_bank: image::Bank<MCUF::Address>,
        must_be_golden: bool,
    ) -> Result<(), Error> {
        let external_flash = if let Some(ref mut external_flash) = self.external_flash {
            external_flash
        } else {
            return Err(Error::NoExternalFlash);
        };

        let input_image = image::image_at(external_flash, input_bank)?;
        duprintln!(
            self.serial,
            "Copying bank {:?} image [Address {:?}, size {:?}] to boot bank.",
            input_bank.index,
            input_image.location().into(),
            input_image.size()
        );
        if must_be_golden && !input_image.is_golden() {
            duprintln!(self.serial, "Image is not golden.",);
            return Err(Error::DeviceError("Image is not golden"));
        }
        let input_image_start_address = input_bank.location;
        let output_image_start_address = output_bank.location;

        // Large transfer buffer ensures that the number of read-write cycles needed
        // to guarantee flash integrity through the process is minimal.
        const TRANSFER_BUFFER_SIZE: usize = KB!(64);
        let mut buffer = [0u8; TRANSFER_BUFFER_SIZE];
        let mut byte_index = 0usize;

        let total_size = input_image.size()
            + SignatureSize::<NistP256>::to_usize()
            + MAGIC_STRING.len()
            + if input_image.is_golden() { GOLDEN_STRING.len() } else { 0 };

        while byte_index < total_size {
            let bytes_to_read = min(TRANSFER_BUFFER_SIZE, total_size.saturating_sub(byte_index));
            block!(external_flash
                .read(input_image_start_address + byte_index, &mut buffer[0..bytes_to_read]))?;
            block!(self
                .mcu_flash
                .write(output_image_start_address + byte_index, &buffer[0..bytes_to_read]))?;
            byte_index += bytes_to_read;
        }
        Ok(())
    }
}
