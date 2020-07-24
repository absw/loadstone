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
use nb::block;

const IMAGE_OFFSET: usize = 4usize;
const TRANSFER_BUFFER_SIZE: usize = 528usize;

pub struct Bootloader<EXTF, MCUF, SRL>
where
    EXTF: flash::ReadWrite,
    MCUF: flash::ReadWrite,
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
    EXTF: flash::ReadWrite,
    MCUF: flash::ReadWrite,
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
        let mut address = EXTF::writable_range().0 + IMAGE_OFFSET;
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

    pub fn test_mcu_flash(&mut self, complex: bool) -> Result<(), Error> {
        if complex {
            Self::test_flash_complex_read_write_cycle(&mut self.mcu_flash)
        } else {
            Self::test_flash_simple_read_write_cycle(&mut self.mcu_flash)
        }
    }

    pub fn test_external_flash(&mut self, complex: bool) -> Result<(), Error> {
        if complex {
            Self::test_flash_complex_read_write_cycle(&mut self.external_flash)
        } else {
            Self::test_flash_simple_read_write_cycle(&mut self.external_flash)
        }
    }

    fn test_flash_simple_read_write_cycle<F>(flash: &mut F) -> Result<(), Error>
    where
        F: flash::ReadWrite,
    {
        let failure = Error::PostError("Flash Read Write cycle failed");
        let mut magic_number_buffer = [0u8; 1];
        let mut new_magic_number_buffer = [0u8; 1];
        let (write_start, _) = F::writable_range();
        let (read_start, _) = F::readable_range();
        block!(flash.read(read_start, &mut magic_number_buffer)).map_err(|_| failure)?;
        new_magic_number_buffer[0] = magic_number_buffer[0].wrapping_add(1);
        block!(flash.write(write_start, &mut new_magic_number_buffer)).map_err(|_| failure)?;
        block!(flash.read(read_start, &mut magic_number_buffer)).map_err(|_| failure)?;
        if magic_number_buffer != new_magic_number_buffer {
            Err(failure)
        } else {
            Ok(())
        }
    }

    fn test_flash_complex_read_write_cycle<F>(flash: &mut F) -> Result<(), Error>
    where
        F: flash::ReadWrite,
    {
        let failure = Error::PostError("Complex Flash Read Write cycle failed");
        let magic_word_buffer = [0xAAu8, 0xBBu8, 0xCCu8, 0xDDu8];
        let superset_byte_buffer = [0xFFu8];
        let expected_final_buffer = [0xFFu8, 0xBBu8, 0xCCu8, 0xDDu8];
        let (write_start, _) = F::writable_range();
        let (read_start, _) = F::readable_range();
        block!(flash.write(write_start, &magic_word_buffer)).map_err(|_| failure)?;
        block!(flash.write(write_start, &superset_byte_buffer)).map_err(|_| failure)?;
        let mut final_buffer = [0x00; 4];
        block!(flash.read(read_start, &mut final_buffer)).map_err(|_| failure)?;
        if expected_final_buffer != final_buffer {
            Err(failure)
        } else {
            Ok(())
        }
    }
}
