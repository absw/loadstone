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
        flash::{self, UnportableDeserialize, UnportableSerialize},
        serial,
    },
    utilities::buffer::TryCollectSlice,
};
use nb::block;

const IMAGE_OFFSET: usize = 4usize;
const TRANSFER_BUFFER_SIZE: usize = 528usize;

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
        let mut cli = self.cli.take().unwrap();
        loop {
            cli.run(&mut self)
        }
    }

    pub fn store_image<I, E>(&mut self, mut bytes: I) -> Result<(), Error>
    where
        I: Iterator<Item = Result<u8, E>>,
    {
        let mut address = EXTF::range().0 + IMAGE_OFFSET;
        let mut buffer = [0u8; TRANSFER_BUFFER_SIZE];
        loop {
            match bytes
                .try_collect_slice(&mut buffer)
                .map_err(|_| Error::DriverError("Serial Read Error"))?
            {
                0 => break Ok(()),
                n => {
                    self.external_flash
                        .write(address, &mut buffer[0..n])
                        .map_err(|_| Error::DriverError("Flash Write Error"))?;
                    address = address + n;
                }
            }
        }
    }

    pub fn format_mcu_flash(&mut self) -> Result<(), Error> {
        block!(self.mcu_flash.erase()).map_err(|_| Error::DriverError("Flash Erase Error"))?;
        block!(image::GlobalHeader::format_default(&mut self.mcu_flash))?;
        for bank in self.mcu_banks {
            block!(image::ImageHeader::format_default(&mut self.mcu_flash, bank.location))?;
        }
        Ok(())
    }

    pub fn format_external_flash(&mut self) -> Result<(), Error> {
        block!(self.mcu_flash.erase()).map_err(|_| Error::DriverError("Flash Erase Error"))?;
        block!(image::GlobalHeader::format_default(&mut self.external_flash))?;
        for bank in self.external_banks {
            block!(image::ImageHeader::format_default(&mut self.external_flash, bank.location))?;
        }
        Ok(())
    }

    pub fn test_mcu_flash(&mut self) -> Result<(), Error> {
        Self::test_flash_read_write_cycle(&mut self.mcu_flash)
    }

    pub fn test_external_flash(&mut self) -> Result<(), Error> {
        Self::test_flash_read_write_cycle(&mut self.external_flash)
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
