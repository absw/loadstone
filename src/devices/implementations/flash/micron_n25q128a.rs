use crate::{
    devices::interfaces::flash::BulkErase,
    hal::{gpio, spi},
};
use nb::block;

const MANUFACTURER_ID: u8 = 0x20;

#[derive(Debug, Clone, Copy)]
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
        self.execute_command(Command::WriteEnable, None, None)?;
        // erase command
        self.execute_command(Command::WriteDisable, None, None)?;
        Ok(())
    }
}

impl<SPI, CS> MicronN25q128a<SPI, CS>
where
    SPI: spi::FullDuplex<u8>,
    CS: gpio::OutputPin,
{
    // Low level helper for executing Micron commands
    fn execute_command(&mut self, command: Command, arguments: Option<&[u8]>, response_buffer: Option<&mut [u8]>) -> Result<(), Error> {
        self.chip_select.set_low();
        block!(self.spi.transmit(Some(command as u8))).map_err(|_| Error::SpiError)?;
        block!(self.spi.receive()).map_err(|_| Error::SpiError)?;

        if let Some(arguments) = arguments {
            for byte in arguments {
                block!(self.spi.transmit(Some(*byte))).map_err(|_| Error::SpiError)?;
                block!(self.spi.receive()).map_err(|_| Error::SpiError)?;
            }
        }

        if let Some(response_buffer) = response_buffer {
            for byte in response_buffer {
                block!(self.spi.transmit(None)).map_err(|_| Error::SpiError)?;
                *byte = block!(self.spi.receive()).map_err(|_| Error::SpiError)?;
            }
        }
        self.chip_select.set_high();
        Ok(())
    }


    fn verify_id(&mut self) -> Result<(), Error> {
        let mut response = [0u8; 1];
        self.execute_command(Command::ReadId, None, Some(&mut response))?;
        match response[0] {
            MANUFACTURER_ID => Ok(()),
            _ => Err(Error::WrongManufacturerId)
        }
    }

    /// Blocks until flash ID read checks out, or until timeout
    pub fn new(spi: SPI, chip_select: CS) -> Result<Self, Error> {
        let mut flash = Self { spi, chip_select };
        flash.verify_id()?;
        Ok(flash)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::hal::mock::{gpio::*, spi::*};

    fn flash_to_test() -> MicronN25q128a<MockSpi::<u8>, MockPin> {
        let mut spi = MockSpi::<u8>::new();
        spi.to_receive.push_back(0);
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
        spi.to_receive.push_back(0);
        spi.to_receive.push_back(WRONG_MANUFACTURER_ID);

        // Then
        assert!(MicronN25q128a::new(spi, MockPin::default()).is_err());

        // Given
        let mut spi = MockSpi::<u8>::new();
        spi.to_receive.push_back(0);
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
