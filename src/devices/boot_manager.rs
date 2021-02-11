//! Fully CLI interactive boot manager for the demo application.

use super::{
    boot_metrics::{boot_metrics, BootMetrics},
    cli::{file_transfer, Cli},
    image,
};
use crate::error::Error;
use blue_hal::{KB, hal::{flash, serial}, stm32pac::SCB, utilities::xmodem};

pub struct BootManager<EXTF, SRL>
where
    EXTF: flash::ReadWrite,
    Error: From<EXTF::Error>,
    SRL: serial::ReadWrite + file_transfer::FileTransfer,
    Error: From<<SRL as serial::Read>::Error>,
{
    pub(crate) external_banks: &'static [image::Bank<<EXTF as flash::ReadWrite>::Address>],
    pub(crate) external_flash: EXTF,
    pub(crate) cli: Option<Cli<SRL>>,
    pub(crate) boot_metrics: Option<BootMetrics>,
}

impl<EXTF, SRL> BootManager<EXTF, SRL>
where
    EXTF: flash::ReadWrite,
    Error: From<EXTF::Error>,
    SRL: serial::ReadWrite + file_transfer::FileTransfer,
    Error: From<<SRL as serial::Read>::Error>,
{
    /// Returns an iterator of all external flash banks.
    pub fn external_banks(&self) -> impl Iterator<Item = image::Bank<EXTF::Address>> {
        self.external_banks.iter().cloned()
    }

    /// Writes a firmware image to an external flash bank.
    pub fn store_image<I>(
        &mut self,
        blocks: I,
        bank: image::Bank<EXTF::Address>,
    ) -> Result<(), Error>
    where
        I: Iterator<Item = [u8; xmodem::PAYLOAD_SIZE]>,
    {
        // A large transfer array ensures minimal flash thrashing as the flash driver
        // has enough information to optimize sector read-write cycles
        const TRANSFER_ARRAY_SIZE: usize = KB!(64);
        assert!(TRANSFER_ARRAY_SIZE % xmodem::PAYLOAD_SIZE == 0);
        let mut transfer_array = [0x00u8; TRANSFER_ARRAY_SIZE];
        let mut memory_index = 0usize;

        for block in blocks {
            let slice = &mut transfer_array[
                (memory_index % TRANSFER_ARRAY_SIZE)
                ..((memory_index % TRANSFER_ARRAY_SIZE) + xmodem::PAYLOAD_SIZE)];
            slice.clone_from_slice(&block);
            memory_index += xmodem::PAYLOAD_SIZE;

            if memory_index % TRANSFER_ARRAY_SIZE == 0 {
                nb::block!(self.external_flash.write(bank.location + (memory_index - TRANSFER_ARRAY_SIZE), &transfer_array))?;
                transfer_array.iter_mut().for_each(|b| *b = 0x00u8);
            }
        }
        let remainder = &transfer_array[0..(memory_index % TRANSFER_ARRAY_SIZE)];
        nb::block!(self.external_flash.write(bank.location + (memory_index - remainder.len()), &remainder))?;
        Ok(())
    }

    pub fn format_external(&mut self) -> Result<(), Error> {
        nb::block!(self.external_flash.erase())?;
        Ok(())
    }

    pub fn reset(&mut self) -> ! { SCB::sys_reset(); }

    pub fn run(mut self, greeting: &'static str) -> ! {
        self.boot_metrics = {
            let metrics = unsafe { boot_metrics().clone() };
            if metrics.is_valid() {
                Some(metrics)
            } else {
                None
            }
        };
        let mut cli = self.cli.take().unwrap();
        loop {
            cli.run(&mut self, greeting)
        }
    }
}
