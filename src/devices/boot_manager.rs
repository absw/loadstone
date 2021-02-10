//! Fully CLI interactive boot manager for the demo application.

use super::{
    boot_metrics::{boot_metrics, BootMetrics},
    cli::{file_transfer, Cli},
    image,
};
use crate::error::Error;
use blue_hal::{
    hal::{flash, serial},
    stm32pac::SCB,
    utilities::xmodem,
};

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
        for (i, block) in blocks.enumerate() {
            nb::block!(self.external_flash.write(bank.location + block.len() * i, &block))?;
        }
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
