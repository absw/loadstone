//! Generic Bootloader.
//!
//! This module contains all bootloader functionality, with
//! the exception of how to construct one. Construction is
//! handled by the `port` module as it depends on board
//! specific information.
use super::image::{self, CRC_SIZE_BYTES, MAGIC_STRING};
use crate::error::Error;
use blue_hal::{duprintln, hal::{flash, serial}, uprintln};
use core::{cmp::min, mem::size_of};
use cortex_m::peripheral::SCB;
use defmt::{error, info};
use nb::block;
use ufmt::{uwrite, uwriteln};

pub struct Bootloader<EXTF, MCUF, SRL>
where
    EXTF: flash::ReadWrite,
    Error: From<EXTF::Error>,
    MCUF: flash::ReadWrite,
    Error: From<MCUF::Error>,
    SRL: serial::ReadWrite,
    Error: From<<SRL as serial::Read>::Error>,
{
    pub(crate) external_flash: EXTF,
    pub(crate) mcu_flash: MCUF,
    pub(crate) external_banks: &'static [image::Bank<<EXTF as flash::ReadWrite>::Address>],
    pub(crate) mcu_banks: &'static [image::Bank<<MCUF as flash::ReadWrite>::Address>],
    pub(crate) serial: SRL,
}

const DEFAULT_BOOT_BANK: u8 = 1;

impl<EXTF, MCUF, SRL> Bootloader<EXTF, MCUF, SRL>
where
    EXTF: flash::ReadWrite,
    Error: From<EXTF::Error>,
    MCUF: flash::ReadWrite,
    Error: From<MCUF::Error>,
    SRL: serial::ReadWrite,
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
        duprintln!(self.serial, "Attempting to boot from default bank");
        match self.boot(DEFAULT_BOOT_BANK).unwrap_err() {
            Error::BankInvalid => duprintln!(self.serial, "Attempted to boot from invalid bank. Restoring image..."),
            Error::BankEmpty => duprintln!(self.serial, "Attempted to boot from empty bank. Restoring image..."),
            Error::CrcInvalid => duprintln!(self.serial, "Crc invalid for stored image. Restoring image..."),
            _ => duprintln!(self.serial, "Unexpected boot error. Restoring image..."),
        };

        match self.restore() {
            Ok(()) => self.boot(DEFAULT_BOOT_BANK).expect("FATAL: Failed to boot from verified image!"),
            Err(e) => {
                duprintln!(self.serial, "Failed to restore.");
                info!("Error: {:?}", e);
                duprintln!(self.serial, "Proceeding to recovery mode...");
                unimplemented!("Recovery Mode");
            }
        }
    }

    /// Restores an image from the preferred external bank. If it fails,
    /// attempts to restore from the golden image.
    fn restore(&mut self) -> Result<(), Error> {
        for bank in 0..self.external_banks.len() {
            if self.copy_image(DEFAULT_BOOT_BANK, bank as u8).is_ok() {
                return Ok(())
            };
        }
        Err(Error::NoImageToRestoreFrom)
    }

    /// Boots into a given memory bank.
    pub fn boot(&mut self, bank_index: u8) -> Result<!, Error> {
        let bank =
            self.mcu_banks.iter().find(|b| b.index == bank_index).ok_or(Error::BankInvalid)?;

        if !bank.bootable {
            return Err(Error::BankInvalid);
        }

        image::image_at(&mut self.mcu_flash, *bank)?;
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
            cortex_m::interrupt::disable();
            (*SCB::ptr()).vtor.write(image_location_raw as u32);
            cortex_m::register::msp::write(initial_stack_pointer);
            reset_handler()
        }
    }

    /// Runs a self test on MCU flash.
    pub fn test_mcu_flash(&mut self) -> Result<(), Error> {
        Self::test_flash_read_write_cycle(&mut self.mcu_flash)
    }

    /// Runs a self test on external flash.
    pub fn test_external_flash(&mut self) -> Result<(), Error> {
        Self::test_flash_read_write_cycle(&mut self.external_flash)
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
    pub fn copy_image(&mut self, input_bank_index: u8, output_bank_index: u8) -> Result<(), Error> {
        let input_bank = self.external_banks[input_bank_index as usize];
        let output_bank = self.mcu_banks[output_bank_index as usize];
        let input_image = image::image_at(&mut self.external_flash, self.external_banks[input_bank_index as usize])?;

        let input_image_start_address = input_bank.location;
        let output_image_start_address = output_bank.location;

        const TRANSFER_BUFFER_SIZE: usize = 2048;
        let mut buffer = [0u8; TRANSFER_BUFFER_SIZE];
        let mut byte_index = 0usize;

        let total_size = input_image.size() + CRC_SIZE_BYTES + MAGIC_STRING.len();
        while byte_index < total_size {
            let bytes_to_read =
                min(TRANSFER_BUFFER_SIZE, total_size.saturating_sub(byte_index));
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

    fn test_flash_read_write_cycle<F>(flash: &mut F) -> Result<(), Error>
    where
        F: flash::ReadWrite,
        Error: From<<F as flash::ReadWrite>::Error>,
    {
        let magic_word_buffer = [0xAAu8, 0xBBu8, 0xCCu8, 0xDDu8];
        let superset_byte_buffer = [0xFFu8];
        let expected_final_buffer = [0xFFu8, 0xBBu8, 0xCCu8, 0xDDu8];
        let (start, _) = flash.range();
        block!(flash.write(start, &magic_word_buffer))?;
        block!(flash.write(start, &superset_byte_buffer))?;
        let mut final_buffer = [0x00; 4];
        block!(flash.read(start, &mut final_buffer))?;
        if expected_final_buffer != final_buffer {
            Err(Error::DriverError("Flash read-write cycle failed"))
        } else {
            Ok(())
        }
    }
}
