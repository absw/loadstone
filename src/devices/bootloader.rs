//! Generic Bootloader.
//!
//! This module contains all bootloader functionality, with
//! the exception of how to construct one. Construction is
//! handled by the `port` module as it depends on board
//! specific information.
use super::image::{self, CRC_SIZE_BYTES, MAGIC_STRING};
use crate::{devices::cli::file_transfer::FileTransfer, error::Error};
use blue_hal::{
    duprintln,
    hal::{flash, serial},
};
use core::{cmp::min, mem::size_of};
use cortex_m::peripheral::SCB;
use defmt::{info, warn};
use nb::block;
use ufmt::uwriteln;

pub struct Bootloader<EXTF, MCUF, SRL>
where
    EXTF: flash::ReadWrite,
    Error: From<EXTF::Error>,
    MCUF: flash::ReadWrite,
    Error: From<MCUF::Error>,
    SRL: serial::ReadWrite,
    Error: From<<SRL as serial::Read>::Error>,
{
    pub(crate) mcu_flash: MCUF,
    pub(crate) external_banks: &'static [image::Bank<<EXTF as flash::ReadWrite>::Address>],
    pub(crate) mcu_banks: &'static [image::Bank<<MCUF as flash::ReadWrite>::Address>],
    pub(crate) external_flash: EXTF,
    pub(crate) serial: SRL,
}

const DEFAULT_BOOT_BANK: u8 = 1;

impl<EXTF, MCUF, SRL> Bootloader<EXTF, MCUF, SRL>
where
    EXTF: flash::ReadWrite,
    Error: From<EXTF::Error>,
    MCUF: flash::ReadWrite,
    Error: From<MCUF::Error>,
    SRL: serial::ReadWrite + serial::TimeoutRead,
    Error: From<<SRL as serial::Read>::Error>,
{
    /// Main bootloader routine. Attempts to boot from MCU image, and
    /// in case of failure proceeds to:
    ///
    /// * Verify selected (main) external bank. If valid, copy to bootable MCU flash bank.
    /// * If main external bank not available or invalid, verify golden image. If valid,
    /// copy to bootable MCU flash bank.
    /// * If golden image not available or invalid, proceed to recovery mode.
    pub fn run(mut self) -> ! {
        assert!(self.external_banks.iter().filter(|b| b.is_golden).count() <= 1);
        assert_eq!(self.mcu_banks.iter().filter(|b| b.is_golden).count(), 0);


        info!("--Loadstone Initialised--");
        self.try_update_image();

        info!("Attempting to boot from default bank.");
        match self.boot(DEFAULT_BOOT_BANK).unwrap_err() {
            Error::BankInvalid => {
                info!("Attempted to boot from invalid bank. Restoring image...")
            }
            Error::BankEmpty => {
                info!("Attempted to boot from empty bank. Restoring image...")
            }
            Error::CrcInvalid => {
                info!("Crc invalid for stored image. Restoring image...")
            }
            _ => info!("Unexpected boot error. Restoring image..."),
        };

        match self.restore() {
            Ok(()) => self.boot(DEFAULT_BOOT_BANK).expect("FATAL: Failed to boot from verified image!"),
            Err(e) => {
                info!("Failed to restore. Error: {:?}", e);
                self.recover();
            }
        }
    }

    /// If the current bootable (MCU flash) image is different from the top
    /// non-golden external image, attempts to replace it.
    fn try_update_image(&mut self) {
        let boot_bank = self.mcu_banks.iter().find(|b| b.index == DEFAULT_BOOT_BANK).unwrap();
        if let Ok(current_image) = image::image_at(&mut self.mcu_flash, *boot_bank) {
            for external_bank in self.external_banks.iter().filter(|b| !b.is_golden) {
                duprintln!(self.serial, "Scanning external bank {:?} for a newer image...", external_bank.index);
                match image::image_at(&mut self.external_flash, *external_bank) {
                    // Using CRC for identification for the time being. Will become
                    // the image's signed hash, which is a valid unique identifier.
                    Ok(image) if image.crc() != current_image.crc() => {
                        duprintln!(self.serial, "Replacing golden image with external bank {:?}...", external_bank.index);
                        self.copy_image(*external_bank, *boot_bank, false).unwrap();
                    },
                    Ok(_image) => break,
                    _ => (),
                }
            };
        }
        duprintln!(self.serial, "No newer image found.");
    }

    /// Restores an image from the preferred external bank. If it fails,
    /// attempts to restore from the golden image.
    fn restore(&mut self) -> Result<(), Error> {
        // Attempt to restore from normal image
        let output = self.mcu_banks.iter().find(|b| b.index == DEFAULT_BOOT_BANK).unwrap();
        for input in self.external_banks.iter().filter(|b| !b.is_golden) {
            if self.copy_image(*input, *output, false).is_ok() {
                return Ok(());
            };
        }

        // Attempt to restore from golden image
        let golden_bank = self.external_banks.iter().find(|b| b.is_golden).unwrap();
        if self.copy_image(*golden_bank, *output, true).is_ok() {
            return Ok(());
        };

        Err(Error::NoImageToRestoreFrom)
    }

    fn recover(&mut self) -> ! {
        duprintln!(self.serial, "-- Loadstone Recovery Mode --");
        duprintln!(self.serial, "Please send golden firmware image via XMODEM.");
        let golden_bank = self.external_banks.iter().find(|b| b.is_golden).unwrap();

        for (i, block) in self.serial.blocks(None).enumerate() {
            nb::block!(self.external_flash.write(golden_bank.location + block.len() * i, &block))
                .map_err(|_| {
                    Error::DriverError("Failed to flash golden image during recovery mode.")
                })
                .unwrap();
        }
        if !image::image_at(&mut self.external_flash, *golden_bank).unwrap().is_golden() {
            duprintln!(self.serial, "FATAL: Flashed image is not a golden image");
        } else {
            duprintln!(self.serial, "Finished flashing golden image.");
        }
        duprintln!(self.serial, "Rebooting...");
        SCB::sys_reset();
    }

    /// Boots into a given memory bank.
    pub fn boot(&mut self, bank_index: u8) -> Result<!, Error> {
        let bank =
            self.mcu_banks.iter().find(|b| b.index == bank_index).ok_or(Error::BankInvalid)?;

        if !bank.bootable {
            return Err(Error::BankInvalid);
        }

        image::image_at(&mut self.mcu_flash, *bank)?.size();
        warn!("Jumping to a new firmware image. This will break `defmt`.");
        let image_location_raw: usize = bank.location.into();

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

    /// Copy from external bank to MCU bank
    pub fn copy_image(
        &mut self,
        input_bank: image::Bank<EXTF::Address>,
        output_bank: image::Bank<MCUF::Address>,
        must_be_golden: bool,
    ) -> Result<(), Error> {
        let input_image = image::image_at(&mut self.external_flash, input_bank)?;
        if must_be_golden && !input_image.is_golden() {
            return Err(Error::DeviceError("Image is not golden"));
        }
        let input_image_start_address = input_bank.location;
        let output_image_start_address = output_bank.location;

        const TRANSFER_BUFFER_SIZE: usize = 2048;
        let mut buffer = [0u8; TRANSFER_BUFFER_SIZE];
        let mut byte_index = 0usize;

        let total_size = input_image.size() + CRC_SIZE_BYTES + MAGIC_STRING.len();
        while byte_index < total_size {
            let bytes_to_read = min(TRANSFER_BUFFER_SIZE, total_size.saturating_sub(byte_index));
            block!(self
                .external_flash
                .read(input_image_start_address + byte_index, &mut buffer[0..bytes_to_read]))?;
            block!(self
                .mcu_flash
                .write(output_image_start_address + byte_index, &buffer[0..bytes_to_read]))?;
            byte_index += bytes_to_read;
        }
        Ok(())
    }
}
