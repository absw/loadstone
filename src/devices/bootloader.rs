//! Generic Bootloader.
//!
//! This module contains all bootloader functionality, with
//! the exception of how to construct one. Construction is
//! handled by the `port` module as it depends on board
//! specific information.
use super::image;
use crate::error::Error;
use blue_hal::hal::flash;
use core::{cmp::min, mem::size_of};
use cortex_m::peripheral::SCB;
use defmt::{error, info};
use nb::block;

pub struct Bootloader<EXTF, MCUF>
where
    EXTF: flash::ReadWrite,
    Error: From<EXTF::Error>,
    MCUF: flash::ReadWrite,
    Error: From<MCUF::Error>,
{
    pub(crate) external_flash: EXTF,
    pub(crate) mcu_flash: MCUF,
    pub(crate) external_banks: &'static [image::Bank<<EXTF as flash::ReadWrite>::Address>],
    pub(crate) mcu_banks: &'static [image::Bank<<MCUF as flash::ReadWrite>::Address>],
}

impl<EXTF, MCUF> Bootloader<EXTF, MCUF>
where
    EXTF: flash::ReadWrite,
    Error: From<EXTF::Error>,
    MCUF: flash::ReadWrite,
    Error: From<MCUF::Error>,
{
    /// Main bootloader routine. Attempts to boot from MCU image, and
    /// in case of failure proceeds to:
    ///
    /// * Verify selected (main) external bank. If valid, copy to bootable MCU flash bank.
    /// * If main external bank not available or invalid, verify golden image. If valid,
    /// copy to bootable MCU flash bank.
    /// * If golden image not available or invalid, proceed to recovery mode.
    pub fn run(mut self) -> ! {
        let default_bank = 1;
        match self.boot(default_bank).unwrap_err() {
            Error::BankInvalid => info!("Attempted to boot from invalid bank. Restoring image..."),
            Error::BankEmpty => info!("Attempted to boot from empty bank. Restoring image..."),
            _ => info!("Unexpected boot error. Restoring image..."),
        };

        match self.restore() {
            Ok(()) => self.boot(default_bank).expect("FATAL: Failed to boot from verified image!"),
            Err(e) => {
                error!("Failed to restore with error: {:?}", e);
                info!("Proceeding to recovery mode...");
                unimplemented!("Recovery Mode");
            }
        }
    }

    /// Restores an image from the preferred external bank. If it fails,
    /// attempts to restore from the golden image.
    fn restore(&mut self) -> Result<(), Error> {
        unimplemented!("Image Restoration");
    }

    /// Boots into a given memory bank.
    pub fn boot(&mut self, bank_index: u8) -> Result<!, Error> {
        let bank =
            self.mcu_banks.iter().find(|b| b.index == bank_index).ok_or(Error::BankInvalid)?;

        if !bank.bootable {
            return Err(Error::BankInvalid);
        }

        unimplemented!();

        //let header = image::ImageHeader::retrieve(&mut self.mcu_flash, &bank)?;
        //if header.size == 0 {
        //    return Err(Error::BankEmpty);
        //}

        //let image_location_raw: usize = bank.location.into();

        //// NOTE(Safety): Thoroughly unsafe operations, for obvious reasons: We are jumping to an
        //// entirely different firmware image! We have to assume everything is at the right place,
        //// or literally anything could happen here. After the interrupts are disabled, there is
        //// no turning back.
        //unsafe {
        //    let initial_stack_pointer = *(image_location_raw as *const u32);
        //    let reset_handler_pointer =
        //        *((image_location_raw + size_of::<u32>()) as *const u32) as *const ();
        //    let reset_handler = core::mem::transmute::<*const (), fn() -> !>(reset_handler_pointer);
        //    cortex_m::interrupt::disable();
        //    (*SCB::ptr()).vtor.write(image_location_raw as u32);
        //    cortex_m::register::msp::write(initial_stack_pointer);
        //    reset_handler()
        //}
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
        unimplemented!();
        //let (input_bank, output_bank) = (
        //    self.external_banks()
        //        .find(|b| b.index == input_bank_index)
        //        .ok_or(Error::BankInvalid)?,
        //    self.mcu_banks().find(|b| b.index == output_bank_index).ok_or(Error::BankInvalid)?,
        //);
        //let input_header = image::ImageHeader::retrieve(&mut self.external_flash, &input_bank)?;
        //if input_header.size > output_bank.size {
        //    return Err(Error::ImageTooBig);
        //} else if input_header.size == 0 {
        //    return Err(Error::BankEmpty);
        //}

        //input_bank.sanity_check(&mut self.external_flash)?;
        //// Output header must be re-formatted before any writing takes place, to ensure
        //// a valid header and an invalid image never coexist.
        //image::ImageHeader::format_default(&mut self.mcu_flash, &output_bank)?;

        //let input_image_start_address = input_bank.location;
        //let output_image_start_address = output_bank.location;

        //let mut buffer = [0u8; TRANSFER_BUFFER_SIZE];
        //let mut byte_index = 0usize;

        //while byte_index < input_header.size {
        //    let bytes_to_read =
        //        min(TRANSFER_BUFFER_SIZE, input_header.size.saturating_sub(byte_index));
        //    block!(self
        //        .external_flash
        //        .read(input_image_start_address + byte_index, &mut buffer[0..bytes_to_read]))?;
        //    block!(self
        //        .mcu_flash
        //        .write(output_image_start_address + byte_index, &buffer[0..bytes_to_read]))?;
        //    byte_index += bytes_to_read;
        //}

        //image::ImageHeader::write(&mut self.mcu_flash, &output_bank, input_header.size)
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
