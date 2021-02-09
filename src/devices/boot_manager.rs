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
        let mut last_index = 0;
        let mut previous_buffer : Option<[u8; xmodem::PAYLOAD_SIZE]> = None;

        for (i, block) in blocks.enumerate() {
            last_index = i;

            previous_buffer = match previous_buffer {
                None => Some(block),
                Some(b) => {
                    let mut output = [0u8; xmodem::PAYLOAD_SIZE * 2];
                    for (source, destination) in b.iter().chain(&block).zip(&mut output) {
                        *destination = *source;
                    }
                    nb::block!(self.external_flash.write(
                        bank.location + xmodem::PAYLOAD_SIZE * i,
                        &output)
                    )?;
                    None
                },
            }
        }

        if let Some(block) = previous_buffer {
            nb::block!(self.external_flash.write(
                bank.location + xmodem::PAYLOAD_SIZE * last_index,
                &block)
            )?;
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
