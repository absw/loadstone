use crate::{
    devices::interfaces::flash::BulkErase,
    hal::{gpio, spi},
};

const MANUFACTURER_ID: u8 = 0x20;

enum Command {
    WriteDisable = 0x04,
    WriteEnable = 0x06,
    ReadId = 0x9E
}

/// Address into the micron chip memory map
pub struct Address(u16);
pub struct Sector(Address);
pub struct Page(Address);
pub struct Word(Address);

pub struct MicronN25q128a<SPI, CS>
where
    SPI: spi::FullDuplex<u8>,
    CS: gpio::OutputPin,
{
    spi: SPI,
    chip_select: CS,
}

#[derive(Debug)]
pub enum Error {
    TimeOut,
    SpiError,
    WrongManufacturerId
}

impl<SPI, CS> BulkErase for MicronN25q128a<SPI, CS>
where
    SPI: spi::FullDuplex<u8>,
    CS: gpio::OutputPin,
{
    type Error = Error;
    fn erase(&mut self) -> nb::Result<(), Self::Error> {
        Self::write_enable(&mut self.spi, gpio::guard_low(&mut self.chip_select))?;
        // erase command
        Self::write_disable(&mut self.spi, gpio::guard_low(&mut self.chip_select))?;
        Ok(())
    }
}

impl<SPI, CS> MicronN25q128a<SPI, CS>
where
    SPI: spi::FullDuplex<u8>,
    CS: gpio::OutputPin,
{
    fn verify_id(spi: &mut SPI, _: gpio::GuardLow) -> Result<(), Error> {
        spi.transmit(Some(Command::ReadId as u8)).map_err(|_| Error::SpiError)?;
        match spi.receive().map_err(|_| Error::SpiError)? {
            MANUFACTURER_ID => Ok(()),
            _ => Err(Error::WrongManufacturerId)
        }
    }

    fn write_enable(spi: &mut SPI, _: gpio::GuardLow) -> Result<(), Error> {
        spi.transmit(Some(Command::WriteEnable as u8)).map_err(|_| Error::SpiError)?;
        spi.receive().map_err(|_| Error::SpiError)?;
        Ok(())
    }

    fn write_disable(spi: &mut SPI, _: gpio::GuardLow) -> Result<(), Error> {
        spi.transmit(Some(Command::WriteDisable as u8)).map_err(|_| Error::SpiError)?;
        spi.receive().map_err(|_| Error::SpiError)?;
        Ok(())
    }

    /// Blocks until flash ID read checks out, or until timeout
    pub fn new(mut spi: SPI, mut chip_select: CS) -> Result<Self, Error> {
        Self::verify_id(&mut spi, gpio::guard_low(&mut chip_select))?;
        let flash = Self { spi, chip_select };
        Ok(flash)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::hal::mock::{gpio::*, spi::*};

    fn flash_to_test() -> MicronN25q128a<MockSpi::<u8>, MockPin> {
        let mut spi = MockSpi::<u8>::new();
        spi.to_receive.push_back(MANUFACTURER_ID);
        MicronN25q128a::new(spi, MockPin::default()).unwrap()
    }

    #[test]
    fn micron_flash_requests_manufacturer_id_on_construction() {
        let MicronN25q128a { mut spi, chip_select } = flash_to_test();

        // Chip select line is wiggled to send command
        assert_eq!(chip_select.changes.len(), 2);
        assert_eq!(chip_select.changes[0], false);
        assert_eq!(chip_select.changes[1], true);

        // Manufacturer ID is requested
        assert_eq!(spi.sent.pop_front().unwrap(), Command::ReadId as u8);
    }

    #[test]
    fn initialisation_succeeds_for_correct_manufacturer_id() {
        const WRONG_MANUFACTURER_ID: u8 = 0x21;

        // Given
        let mut spi = MockSpi::<u8>::new();
        spi.to_receive.push_back(WRONG_MANUFACTURER_ID);

        // Then
        assert!(MicronN25q128a::new(spi, MockPin::default()).is_err());

        // Given
        let mut spi = MockSpi::<u8>::new();
        spi.to_receive.push_back(MANUFACTURER_ID);

        // Then
        assert!(MicronN25q128a::new(spi, MockPin::default()).is_ok());
    }

    #[test]
    fn bulk_erase_sets_write_enable() {
        // Given
        let mut flash = flash_to_test();
        flash.spi.sent.clear();

        // When
        flash.erase().unwrap();

        // Then
        assert_eq!(flash.spi.sent[0], Command::WriteEnable as u8);
    }
}
