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

use defmt::info;

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
        size: usize,
        bank: image::Bank<EXTF::Address>,
    ) -> Result<(), Error>
    where
        I: Iterator<Item = [u8; xmodem::PAYLOAD_SIZE]>,
    {
        if size > bank.size {
            return Err(Error::ImageTooBig);
        }

        unimplemented!();
        //// Header must be re-formatted before any writing takes place, to ensure
        //// a valid header and an invalid image never coexist.
        //image::ImageHeader::format_default(&mut self.external_flash, &bank)?;

        //let mut buffer = [0u8; TRANSFER_BUFFER_SIZE];
        //let address = bank.location;
        //let mut bytes_written = 0;

        //let mut bytes = blocks.flat_map(|b| IntoIter::new(b));

        //loop {
        //    let distance_to_end = size - bytes_written;
        //    let received = bytes.collect_slice(&mut buffer);
        //    if received == 0 {
        //        break;
        //    }
        //    let bytes_to_write = min(distance_to_end, received);
        //    block!(self.external_flash.write(address + bytes_written, &buffer[0..bytes_to_write]))?;
        //    bytes_written += bytes_to_write;
        //}

        //if bytes_written == size {
        //    image::ImageHeader::write(&mut self.external_flash, &bank, size)
        //} else {
        //    Err(Error::NotEnoughData)
        //}
    }

    pub fn reset(&mut self) -> ! { SCB::sys_reset(); }

    pub fn run(mut self) -> ! {
        let mut cli = self.cli.take().unwrap();
        loop {
            info!("Starting CLI");
            cli.run(&mut self)
        }
    }
}
