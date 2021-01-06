//! Generic Bootloader.
//!
//! This module contains all bootloader functionality, with
//! the exception of how to construct one. Construction is
//! handled by the `port` module as it depends on board
//! specific information.
use super::image;
use crate::{
    devices::cli::Cli,
    error::Error,
    hal::{flash, serial},
    utilities::buffer::TryCollectSlice,
};
use core::{cmp::min, mem::size_of};
use cortex_m::peripheral::SCB;
use image::TRANSFER_BUFFER_SIZE;
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
    pub(crate) external_flash: EXTF,
    pub(crate) mcu_flash: MCUF,
    pub(crate) cli: Option<Cli<SRL>>,
    pub(crate) external_banks: &'static [image::Bank<<EXTF as flash::ReadWrite>::Address>],
    pub(crate) mcu_banks: &'static [image::Bank<<MCUF as flash::ReadWrite>::Address>],
}

impl<EXTF, MCUF, SRL> Bootloader<EXTF, MCUF, SRL>
where
    EXTF: flash::ReadWrite,
    Error: From<EXTF::Error>,
    MCUF: flash::ReadWrite,
    Error: From<MCUF::Error>,
    SRL: serial::ReadWrite,
    Error: From<<SRL as serial::Read>::Error>,
{
    /// Runs the CLI.
    pub fn run(mut self) -> ! {
        let mut cli = self.cli.take().unwrap();
        loop { cli.run(&mut self) }
    }

    /// Writes a firmware image to an external flash bank.
    pub fn store_image<I, E>(
        &mut self,
        mut bytes: I,
        size: usize,
        bank: image::Bank<EXTF::Address>,
    ) -> Result<(), Error>
    where
        I: Iterator<Item = Result<u8, E>>,
        Error: From<E>,
    {
        if size > bank.size {
            return Err(Error::ImageTooBig);
        }

        // Header must be re-formatted before any writing takes place, to ensure
        // a valid header and an invalid image never coexist.
        image::ImageHeader::format_default(&mut self.external_flash, &bank)?;

        let mut address = bank.location;
        let mut buffer = [0u8; TRANSFER_BUFFER_SIZE];
        loop {
            match bytes.try_collect_slice(&mut buffer)? {
                0 => break,
                n => {
                    block!(self.external_flash.write(address, &buffer[0..n]))?;
                    address = address + n;
                }
            }
        }

        image::ImageHeader::write(&mut self.external_flash, &bank, size)
    }

    pub fn reset(&mut self) -> ! {
        SCB::sys_reset();
    }

    /// Boots into a given memory bank.
    pub fn boot(
        mcu_flash: &mut MCUF, mcu_banks: &'static [image::Bank<<MCUF>::Address>], bank_index: u8
    ) -> Result<!, Error> {
        let bank =
            mcu_banks.iter().find(|b| b.index == bank_index).ok_or(Error::BankInvalid)?;

        if !bank.bootable {
            return Err(Error::BankInvalid);
        }

        let header = image::ImageHeader::retrieve(mcu_flash, &bank)?;
        if header.size == 0 {
            return Err(Error::BankEmpty);
        }

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

    /// Formats all MCU flash banks.
    pub fn format_mcu_flash(&mut self) -> Result<(), Error> {
        // Headers must be formatted first before the full flash
        // erase, to ensure no half-formatted state in case of restart
        image::GlobalHeader::format_default(&mut self.mcu_flash)?;
        for bank in self.mcu_banks {
            image::ImageHeader::format_default(&mut self.mcu_flash, bank)?;
        }
        block!(self.mcu_flash.erase())?;
        image::GlobalHeader::format_default(&mut self.mcu_flash)?;
        for bank in self.mcu_banks {
            image::ImageHeader::format_default(&mut self.mcu_flash, bank)?;
        }
        Ok(())
    }

    /// Formats all external flash banks.
    pub fn format_external_flash(&mut self) -> Result<(), Error> {
        // Headers must be formatted first before the full flash
        // erase, to ensure no half-formatted state in case of restart
        image::GlobalHeader::format_default(&mut self.external_flash)?;
        for bank in self.external_banks {
            image::ImageHeader::format_default(&mut self.external_flash, bank)?;
        }
        block!(self.external_flash.erase())?;
        image::GlobalHeader::format_default(&mut self.external_flash)?;
        for bank in self.external_banks {
            image::ImageHeader::format_default(&mut self.external_flash, bank)?;
        }
        Ok(())
    }

    /// Runs a self test on MCU flash.
    pub fn test_mcu_flash(&mut self) -> Result<(), Error> {
        Self::test_flash_read_write_cycle(&mut self.mcu_flash)
    }

    /// Runs a self test on external flash.
    pub fn test_external_flash(&mut self) -> Result<(), Error> {
        Self::test_flash_read_write_cycle(&mut self.external_flash)
    }

    /// Finds and returns the image header of a given bank index.
    pub fn image_at_bank(&mut self, index: u8) -> Option<image::ImageHeader> {
        let mcu_bank = self.mcu_banks().find(|b| b.index == index);
        let external_bank = self.external_banks().find(|b| b.index == index);

        let image = if let Some(bank) = mcu_bank {
            image::ImageHeader::retrieve(&mut self.mcu_flash, &bank).ok()
        } else if let Some(bank) = external_bank {
            image::ImageHeader::retrieve(&mut self.external_flash, &bank).ok()
        } else {
            None
        };

        match image {
            Some(image) if image.size > 0 => Some(image),
            _ => None,
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
    pub fn copy_image(&mut self, input_bank_index: u8, output_bank_index: u8) -> Result<(), Error> {
        let (input_bank, output_bank) = (
            self.external_banks()
                .find(|b| b.index == input_bank_index)
                .ok_or(Error::BankInvalid)?,
            self.mcu_banks().find(|b| b.index == output_bank_index).ok_or(Error::BankInvalid)?,
        );
        let input_header = image::ImageHeader::retrieve(&mut self.external_flash, &input_bank)?;
        if input_header.size > output_bank.size {
            return Err(Error::ImageTooBig);
        } else if input_header.size == 0 {
            return Err(Error::BankEmpty);
        }

        input_bank.sanity_check(&mut self.external_flash)?;
        // Output header must be re-formatted before any writing takes place, to ensure
        // a valid header and an invalid image never coexist.
        image::ImageHeader::format_default(&mut self.mcu_flash, &output_bank)?;

        let input_image_start_address = input_bank.location;
        let output_image_start_address = output_bank.location;

        let mut buffer = [0u8; TRANSFER_BUFFER_SIZE];
        let mut byte_index = 0usize;

        while byte_index < input_header.size {
            let bytes_to_read =
                min(TRANSFER_BUFFER_SIZE, input_header.size.saturating_sub(byte_index));
            block!(self
                .external_flash
                .read(input_image_start_address + byte_index, &mut buffer[0..bytes_to_read]))?;
            block!(self
                .mcu_flash
                .write(output_image_start_address + byte_index, &buffer[0..bytes_to_read]))?;
            byte_index += bytes_to_read;
        }

        image::ImageHeader::write(&mut self.mcu_flash, &output_bank, input_header.size)
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
            Err(Error::DriverError("Flash Read Write cycle failed"))
        } else {
            Ok(())
        }
    }
}
