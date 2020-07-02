use crate::{
    devices::interfaces::flash::BulkErase,
    hal::{gpio, spi},
    utilities::bitwise::BitFlags,
};
use nb::block;

const MANUFACTURER_ID: u8 = 0x20;

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

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    TimeOut,
    SpiError,
    WrongManufacturerId,
}

#[derive(Debug, Clone, Copy)]
enum Command {
    WriteDisable = 0x04,
    ReadStatus = 0x05,
    WriteEnable = 0x06,
    ReadId = 0x9E,
    BulkErase = 0xC7,
}

struct Status {
    write_in_progress: bool,
}

impl<SPI, CS> BulkErase for MicronN25q128a<SPI, CS>
where
    SPI: spi::FullDuplex<u8>,
    CS: gpio::OutputPin,
{
    type Error = Error;
    fn erase(&mut self) -> nb::Result<(), Self::Error> {
        // Early yield if flash is not ready for writing
        if !self.can_write()? {
            Err(nb::Error::WouldBlock)
        } else {
            self.execute_command(Command::WriteEnable, None, None)?;
            self.execute_command(Command::BulkErase, None, None)?;
            self.execute_command(Command::WriteDisable, None, None)?;
            Ok(())
        }
    }
}

impl<SPI, CS> MicronN25q128a<SPI, CS>
where
    SPI: spi::FullDuplex<u8>,
    CS: gpio::OutputPin,
{
    fn can_write(&mut self) -> nb::Result<bool, Error> {
        let status = self.status()?;
        Ok(!status.write_in_progress)
    }

    // Low level helper for executing Micron commands
    fn execute_command(
        &mut self, command: Command, arguments: Option<&[u8]>, response_buffer: Option<&mut [u8]>,
    ) -> nb::Result<(), Error> {
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

    fn verify_id(&mut self) -> nb::Result<(), Error> {
        let mut response = [0u8; 1];
        self.execute_command(Command::ReadId, None, Some(&mut response))?;
        match response[0] {
            MANUFACTURER_ID => Ok(()),
            _ => Err(nb::Error::Other(Error::WrongManufacturerId)),
        }
    }

    fn status(&mut self) -> nb::Result<Status, Error> {
        let mut response = [0u8; 1];
        self.execute_command(Command::ReadStatus, None, Some(&mut response))?;
        let response = response[0];
        Ok(Status { write_in_progress: response.is_set(0) })
    }

    /// Blocks until flash ID read checks out, or until timeout
    pub fn new(spi: SPI, chip_select: CS) -> nb::Result<Self, Error> {
        let mut flash = Self { spi, chip_select };
        flash.verify_id()?;
        Ok(flash)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::hal::mock::{gpio::*, spi::*};

    fn flash_to_test() -> MicronN25q128a<MockSpi<u8>, MockPin> {
        let mut spi = MockSpi::<u8>::new();
        spi.to_receive.push_back(0);
        spi.to_receive.push_back(MANUFACTURER_ID);
        let pin = MockPin::default();
        let mut flash = MicronN25q128a::new(spi, pin).unwrap();
        // Chip select line is wiggled to send command
        assert_eq!(flash.chip_select.changes.len(), 2);
        assert_eq!(flash.chip_select.changes[0], false);
        assert_eq!(flash.chip_select.changes[1], true);
        assert_eq!(flash.spi.sent.pop_front().unwrap(), Command::ReadId as u8);
        flash.spi.sent.clear();
        flash.spi.to_receive.clear();
        flash
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
    fn bulk_erase_sets_write_enable_writes_command_and_sets_write_disable() {
        // Given
        let mut flash = flash_to_test();

        // When
        flash.erase().unwrap();

        // Then
        assert_eq!(flash.spi.sent[0], Command::ReadStatus as u8);
        assert_eq!(flash.spi.sent[1], Command::WriteEnable as u8);
        assert_eq!(flash.spi.sent[2], Command::BulkErase as u8);
        assert_eq!(flash.spi.sent[3], Command::WriteDisable as u8);
    }

    #[test]
    fn write_capable_commands_yield_if_device_busy() {
        // Given
        const BUSY_WRITING_STATUS: u8 = 1;
        let mut flash = flash_to_test();
        flash.spi.to_receive.push_back(0);
        flash.spi.to_receive.push_back(BUSY_WRITING_STATUS);

        // Then
        assert_eq!(flash.erase(), Err(nb::Error::WouldBlock));
    }
}
