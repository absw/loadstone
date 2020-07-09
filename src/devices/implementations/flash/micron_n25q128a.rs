use crate::{
    devices::interfaces::flash::{BulkErase, Read, Write},
    hal::qspi,
    utilities::bitwise::BitFlags,
};
use nb::block;

const MANUFACTURER_ID: u8 = 0x20;

/// Address into the micron chip memory map
#[derive(Clone, Copy, Debug)]
pub struct Address(pub u32);

pub struct MicronN25q128a<QSPI>
where
    QSPI: qspi::Indirect,
{
    qspi: QSPI,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    TimeOut,
    QspiError,
    WrongManufacturerId,
    MisalignedAccess,
}

#[derive(Debug, Clone, Copy)]
enum Command {
    PageProgram = 0x02,
    Read = 0x03,
    WriteDisable = 0x04,
    ReadStatus = 0x05,
    WriteEnable = 0x06,
    SubsectorErase = 0x20,
    ReadId = 0x9E,
    BulkErase = 0xC7,
}

struct Status {
    write_in_progress: bool,
}

enum CommandData<'a> {
    _Arguments(&'a [u8]),
    Read(&'a mut [u8]),
    Write(&'a [u8]),
    None,
}

impl<QSPI> BulkErase for MicronN25q128a<QSPI>
where
    QSPI: qspi::Indirect,
{
    type Error = Error;
    fn erase(&mut self) -> nb::Result<(), Self::Error> {
        // Early yield if flash is not ready for writing
        if self.status()?.write_in_progress {
            Err(nb::Error::WouldBlock)
        } else {
            self.execute_command(Command::WriteEnable, None, CommandData::None)?;
            self.execute_command(Command::BulkErase, None, CommandData::None)?;
            self.execute_command(Command::WriteDisable, None, CommandData::None)?;
            Ok(())
        }
    }
}

impl<QSPI> Write<Address> for MicronN25q128a<QSPI>
where
    QSPI: qspi::Indirect,
{
    type Error = Error;

    fn write(&mut self, address: Address, bytes: &[u8]) -> nb::Result<(), Self::Error> {
        // TODO remove page alignment limitations
        if address.0 % 256 != 0 || bytes.len() > 256 {
            return Err(nb::Error::Other(Error::MisalignedAccess));
        }

        // TODO read subsector first before erasing (to preserve previous values)
        self.execute_command(Command::WriteEnable, None, CommandData::None)?;
        self.execute_command(Command::SubsectorErase, Some(address), CommandData::None)?;
        while self.status()?.write_in_progress {}
        self.execute_command(Command::WriteEnable, None, CommandData::None)?;
        self.execute_command(Command::PageProgram, Some(address), CommandData::Write(&bytes))?;
        Ok(())
    }
}

impl<QSPI> Read<Address> for MicronN25q128a<QSPI>
where
    QSPI: qspi::Indirect,
{
    type Error = Error;
    fn read(&mut self, address: Address, bytes: &mut [u8]) -> nb::Result<(), Self::Error> {
        if self.status()?.write_in_progress {
            Err(nb::Error::WouldBlock)
        } else {
            self.execute_command(Command::Read, Some(address), CommandData::Read(bytes))
        }
    }
}

impl<QSPI> MicronN25q128a<QSPI>
where
    QSPI: qspi::Indirect,
{
    // Low level helper for executing Micron commands
    fn execute_command(
        &mut self,
        command: Command,
        address: Option<Address>,
        data: CommandData,
    ) -> nb::Result<(), Error> {
        match data {
            CommandData::_Arguments(buffer) => {
                block!(self.qspi.write(Some(command as u8), address.map(|a| a.0), Some(buffer), 0))
            }
            CommandData::Write(buffer) => {
                block!(self.qspi.write(Some(command as u8), address.map(|a| a.0), Some(buffer), 0))
            }
            CommandData::Read(buffer) => {
                block!(self.qspi.read(Some(command as u8), address.map(|a| a.0), buffer, 0))
            }
            CommandData::None => {
                block!(self.qspi.write(Some(command as u8), address.map(|a| a.0), None, 0))
            }
        }
        .map_err(|_| nb::Error::Other(Error::QspiError))
    }

    fn verify_id(&mut self) -> nb::Result<(), Error> {
        let mut response = [0u8; 1];
        self.execute_command(Command::ReadId, None, CommandData::Read(&mut response))?;
        match response[0] {
            MANUFACTURER_ID => Ok(()),
            _ => Err(nb::Error::Other(Error::WrongManufacturerId)),
        }
    }

    fn status(&mut self) -> nb::Result<Status, Error> {
        let mut response = [0u8; 1];
        self.execute_command(Command::ReadStatus, None, CommandData::Read(&mut response))?;
        let response = response[0];
        Ok(Status { write_in_progress: response.is_set(0) })
    }

    /// Blocks until flash ID read checks out, or until timeout
    pub fn new(qspi: QSPI) -> nb::Result<Self, Error> {
        let mut flash = Self { qspi };
        flash.verify_id()?;
        Ok(flash)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::hal::mock::{gpio::*, qspi::*};

    fn flash_to_test() -> MicronN25q128a<MockQspi> {
        let mut qspi = MockQspi::default();
        qspi.to_read.push_back(vec![MANUFACTURER_ID]);
        let mut flash = MicronN25q128a::new(qspi).unwrap();
        let initial_read = flash.qspi.read_records[0].clone();
        assert_eq!(initial_read.instruction, Some(Command::ReadId as u8));
        flash.qspi.clear();
        flash
    }

    #[test]
    fn initialisation_succeeds_for_correct_manufacturer_id() {
        const WRONG_MANUFACTURER_ID: u8 = 0x21;
        let mut qspi = MockQspi::default();
        qspi.to_read.push_back(vec![WRONG_MANUFACTURER_ID]);

        // Then
        assert!(MicronN25q128a::new(qspi).is_err());

        // Given
        let mut qspi = MockQspi::default();
        qspi.to_read.push_back(vec![MANUFACTURER_ID]);

        // Then
        assert!(MicronN25q128a::new(qspi).is_ok());
    }

    #[test]
    fn bulk_erase_sets_write_enable_writes_command_and_sets_write_disable() {
        // Given
        let mut flash = flash_to_test();

        // When
        flash.erase().unwrap();

        // Then
        assert_eq!(flash.qspi.read_records[0].instruction, Some(Command::ReadStatus as u8));
        assert_eq!(flash.qspi.write_records[0].instruction, Some(Command::WriteEnable as u8));
        assert_eq!(flash.qspi.write_records[1].instruction, Some(Command::BulkErase as u8));
        assert_eq!(flash.qspi.write_records[2].instruction, Some(Command::WriteDisable as u8));
    }

    #[test]
    fn write_capable_commands_yield_if_device_busy() {
        // Given
        const BUSY_WRITING_STATUS: u8 = 1;
        let mut flash = flash_to_test();
        flash.qspi.to_read.push_back(vec![BUSY_WRITING_STATUS]);

        // Then
        assert_eq!(flash.erase(), Err(nb::Error::WouldBlock));
    }

    #[test]
    fn page_program_command_sequence() {
        // Given
        let mut flash = flash_to_test();
        let address = Address(0x0000);
        let data = [0xAAu8; 256];

        // When
        flash.write(address, &data).unwrap();

        // Then
        assert_eq!(flash.qspi.read_records[0].instruction, Some(Command::ReadStatus as u8));
        assert_eq!(flash.qspi.write_records[0].instruction, Some(Command::WriteEnable as u8));
        assert_eq!(flash.qspi.write_records[1].instruction, Some(Command::SubsectorErase as u8));
        assert_eq!(flash.qspi.write_records[2].instruction, Some(Command::WriteEnable as u8));
        assert_eq!(flash.qspi.write_records[3].instruction, Some(Command::PageProgram as u8));
    }
}
