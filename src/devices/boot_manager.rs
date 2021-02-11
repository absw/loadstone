//! Fully CLI interactive boot manager for the demo application.

use super::{
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
        use crate::utility::*;

        let mut buffer = [0u8; xmodem::PAYLOAD_SIZE * 2];
        let buffer_slice = buffer.as_mut_slice();
        for (i, block) in blocks.pairs().enumerate() {
            let view = match block {
                Pair::One(a) => {
                    buffer_slice[..xmodem::PAYLOAD_SIZE].copy_from_slice(&a);
                    &buffer_slice[..xmodem::PAYLOAD_SIZE]
                },
                Pair::Two(a, b) => {
                    buffer_slice[..xmodem::PAYLOAD_SIZE].copy_from_slice(&a);
                    buffer_slice[xmodem::PAYLOAD_SIZE..].copy_from_slice(&b);
                    &buffer_slice[..]
                },
            };
            nb::block!(self.external_flash.write(
                bank.location + (i * xmodem::PAYLOAD_SIZE * 2),
                view
            ))?;
        }
        Ok(())
    }

    pub fn format_external(&mut self) -> Result<(), Error> {
        nb::block!(self.external_flash.erase())?;
        Ok(())
    }

    pub fn reset(&mut self) -> ! { SCB::sys_reset(); }

    pub fn run(mut self, greeting: &'static str) -> ! {
        let mut cli = self.cli.take().unwrap();
        loop {
            cli.run(&mut self, greeting)
        }
    }
}
