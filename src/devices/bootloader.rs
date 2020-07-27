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
    hal::{
        flash,
        serial,
    },
    utilities::buffer::TryCollectSlice,
};
use core::{cmp::min, mem::size_of};
use cortex_m::interrupt;
use nb::block;

const TRANSFER_BUFFER_SIZE: usize = 2048usize;

pub struct Bootloader<EXTF, MCUF, SRL>
where
    EXTF: flash::ReadWrite + flash::BulkErase,
    MCUF: flash::ReadWrite + flash::BulkErase,
    SRL: serial::ReadWrite,
{
    pub(crate) external_flash: EXTF,
    pub(crate) mcu_flash: MCUF,
    pub(crate) cli: Option<Cli<SRL>>,
    pub(crate) external_banks: &'static [image::Bank<<EXTF as flash::ReadWrite>::Address>],
    pub(crate) mcu_banks: &'static [image::Bank<<MCUF as flash::ReadWrite>::Address>],
}

impl<EXTF, MCUF, SRL> Bootloader<EXTF, MCUF, SRL>
where
    EXTF: flash::ReadWrite + flash::BulkErase,
    MCUF: flash::ReadWrite + flash::BulkErase,
    SRL: serial::ReadWrite,
{
    pub fn run(mut self) -> ! {
        // Basic runtime sanity checks: all bank indices must be sequential starting from MCU
        let indices =
            self.mcu_banks().map(|b| b.index).chain(self.external_banks().map(|b| b.index));
        assert!((1..).zip(indices).all(|(a, b)| a == b));

        // Decouple the CLI to facilitate passing mutable references to the bootloader to it.
        let mut cli = self.cli.take().unwrap();
        loop {
            cli.run(&mut self)
        }
    }

    pub fn store_image<I, E>(
        &mut self,
        mut bytes: I,
        size: usize,
        bank_index: u8,
    ) -> Result<(), Error>
    where
        I: Iterator<Item = Result<u8, E>>,
    {
        let bank = self
            .external_banks()
            .find(|b| b.index == bank_index)
            .ok_or(Error::LogicError("Bank Not Found"))?;

        if size > bank.size {
            return Err(Error::LogicError("Flash image too big for bank."));
        }

        let mut address = bank.location;
        let mut buffer = [0u8; TRANSFER_BUFFER_SIZE];
        loop {
            match bytes
                .try_collect_slice(&mut buffer)
                .map_err(|_| Error::DriverError("Serial Read Error"))?
            {
                0 => break,
                n => {
                    self.external_flash
                        .write(address, &mut buffer[0..n])
                        .map_err(|_| Error::DriverError("Flash Write Error"))?;
                    address = address + n;
                }
            }
        }

        block!(image::ImageHeader::write(&mut self.external_flash, &bank, size, 0u32))
    }

    pub fn boot(&mut self, bank_index: u8) -> Result<!, Error> {
        let bank = self
            .mcu_banks
            .iter()
            .find(|b| b.index == bank_index)
            .ok_or(Error::LogicError("Bank doesn't exist or isn't in MCU"))?;

        if !bank.bootable {
            return Err(Error::LogicError("Bank is not bootable!"));
        }

        let header = block!(image::ImageHeader::retrieve(&mut self.mcu_flash, &bank))?;
        if header.size == 0 {
            return Err(Error::LogicError("Image is empty"));
        }

        let image_location_raw: usize = bank.location.into();

        // NOTE(Safety): Thoroughly unsafe operations, for obvious reasons: We are jumping to an
        // entirely different firmware image! We have to assume everything is at the right place,
        // or literally anything could happen here. After the interrupts are disabled, there is
        // no turning back.
        unsafe {
            let initial_stack_pointer =  *(image_location_raw as *const u32);
            let reset_handler_pointer = *((image_location_raw + size_of::<u32>()) as *const u32) as *const ();
            let reset_handler = core::mem::transmute::<*const (), fn() -> !>(reset_handler_pointer);
            cortex_m::interrupt::disable();
            cortex_m::register::msp::write(initial_stack_pointer);
            reset_handler()
        }
    }

    pub fn format_mcu_flash(&mut self) -> Result<(), Error> {
        block!(self.mcu_flash.erase()).map_err(|_| Error::DriverError("Flash Erase Error"))?;
        block!(image::GlobalHeader::format_default(&mut self.mcu_flash))?;
        for bank in self.mcu_banks {
            block!(image::ImageHeader::format_default(&mut self.mcu_flash, bank))?;
        }
        Ok(())
    }

    pub fn format_external_flash(&mut self) -> Result<(), Error> {
        block!(self.mcu_flash.erase()).map_err(|_| Error::DriverError("Flash Erase Error"))?;
        block!(image::GlobalHeader::format_default(&mut self.external_flash))?;
        for bank in self.external_banks {
            block!(image::ImageHeader::format_default(&mut self.external_flash, bank))?;
        }
        Ok(())
    }

    pub fn test_mcu_flash(&mut self) -> Result<(), Error> {
        Self::test_flash_read_write_cycle(&mut self.mcu_flash)
    }

    pub fn test_external_flash(&mut self) -> Result<(), Error> {
        Self::test_flash_read_write_cycle(&mut self.external_flash)
    }

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

    pub fn mcu_banks(&self) -> impl Iterator<Item = image::Bank<MCUF::Address>> {
        self.mcu_banks.iter().cloned()
    }

    pub fn external_banks(&self) -> impl Iterator<Item = image::Bank<EXTF::Address>> {
        self.external_banks.iter().cloned()
    }

    /// Copy from external bank to MCU bank
    pub fn copy_image(&mut self, input_bank_index: u8, output_bank_index: u8) -> Result<(), Error> {
        let (input_bank, output_bank) = (
            self.external_banks()
                .find(|b| b.index == input_bank_index)
                .ok_or(Error::LogicError("Input bank doesn't exist or isn't external."))?,
            self.mcu_banks()
                .find(|b| b.index == output_bank_index)
                .ok_or(Error::LogicError("Output bank doesn't exist or isn't in MCU"))?,
        );
        let input_header =
            block!(image::ImageHeader::retrieve(&mut self.external_flash, &input_bank))?;
        if input_header.size > output_bank.size {
            return Err(Error::LogicError("Image doesn't fit in output bank"));
        } else if input_header.size == 0 {
            return Err(Error::LogicError("Input image is empty"));
        }

        let input_image_start_address = input_bank.location;
        let output_image_start_address = output_bank.location;

        let mut buffer = [0u8; TRANSFER_BUFFER_SIZE];
        let mut byte_index = 0usize;

        while byte_index < input_header.size {
            let bytes_to_read =
                min(TRANSFER_BUFFER_SIZE, input_header.size.saturating_sub(byte_index));
            block!(self
                .external_flash
                .read(input_image_start_address + byte_index, &mut buffer[0..bytes_to_read]))
            .map_err(|_| Error::DriverError("Error reading image from external flash"))?;
            block!(self
                .mcu_flash
                .write(output_image_start_address + byte_index, &buffer[0..bytes_to_read]))
            .map_err(|_| Error::DriverError("Error writing image to mcu flash"))?;
            byte_index += bytes_to_read;
        }

        block!(image::ImageHeader::write(
            &mut self.mcu_flash,
            &output_bank,
            input_header.size,
            input_header.crc
        ))
        .map_err(|_| Error::DriverError("Error writing header to mcu flash"))
    }

    fn test_flash_read_write_cycle<F>(flash: &mut F) -> Result<(), Error>
    where
        F: flash::ReadWrite,
    {
        let failure = Error::PostError("Flash Read Write cycle failed");
        let magic_word_buffer = [0xAAu8, 0xBBu8, 0xCCu8, 0xDDu8];
        let superset_byte_buffer = [0xFFu8];
        let expected_final_buffer = [0xFFu8, 0xBBu8, 0xCCu8, 0xDDu8];
        let (start, _) = F::range();
        block!(flash.write(start, &magic_word_buffer)).map_err(|_| failure)?;
        block!(flash.write(start, &superset_byte_buffer)).map_err(|_| failure)?;
        let mut final_buffer = [0x00; 4];
        block!(flash.read(start, &mut final_buffer)).map_err(|_| failure)?;
        if expected_final_buffer != final_buffer {
            Err(failure)
        } else {
            Ok(())
        }
    }
}
