use crate::{devices::interfaces::flash::Erase, hal::spi};

/// Address into the micron chip memory map
pub struct Address(u16);
pub struct Sector(Address);
pub struct Page(Address);
pub struct Word(Address);

pub struct MicronN25q128a<SPI>
where
    SPI: spi::FullDuplex<u8>,
{
    spi: SPI,
}

pub struct Error {}

impl<SPI> Erase<Sector> for MicronN25q128a<SPI>
where
    SPI: spi::FullDuplex<u8>,
{
    type Error = Error;
    fn erase(sector: Sector) -> nb::Result<(), Self::Error> {
        unimplemented!();
    }
}
