//! Device driver for the [Micron N24q128a](../../../../../../../../documentation/hardware/micron_flash.pdf#page=0)
use crate::{
    hal::{
        flash::{BulkErase, Read, Write},
        qspi, time,
    },
    utilities::bitwise::BitFlags,
};
use crate::error::Error as BootloaderError;
use core::marker::PhantomData;
use nb::block;

/// From [datasheet table 19](../../../../../../../../documentation/hardware/micron_flash.pdf#page=37)
const MANUFACTURER_ID: u8 = 0x20;

/// Address into the micron chip [memory map](../../../../../../../../documentation/hardware/micron_flash.pdf#page=14)
#[derive(Clone, Copy, Debug)]
pub struct Address(pub u32);

/// MicronN25q128a driver, generic over a QSPI programmed in indirect mode
pub struct MicronN25q128a<QSPI, NOW, I>
where
    QSPI: qspi::Indirect,
    NOW: time::Now<I>,
    I: time::Instant,
{
    qspi: QSPI,
    timeout: Option<(time::Milliseconds, NOW)>,
    _marker: PhantomData<I>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    TimeOut,
    QspiError,
    WrongManufacturerId,
    MisalignedAccess,
}

impl From<Error> for BootloaderError {
    fn from(error: Error) -> Self {
        match error {
            Error::TimeOut => BootloaderError::DriverError("Micron n25q128a timed out"),
            Error::QspiError => {
                BootloaderError::DriverError("Micron n25q128a QSPI access error")
            }
            Error::WrongManufacturerId => {
                BootloaderError::DriverError("Micron n25q128a reported wrong manufacturer ID")
            }
            Error::MisalignedAccess => {
                BootloaderError::DriverError("Misaligned access to Micron n25q128a requested")
            }
        }
    }
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
    _write_enable_latch: bool,
}

enum CommandData<'a> {
    Read(&'a mut [u8]),
    Write(&'a [u8]),
    None,
}

impl<QSPI, NOW, I> BulkErase for MicronN25q128a<QSPI, NOW, I>
where
    QSPI: qspi::Indirect,
    NOW: time::Now<I>,
    I: time::Instant,
{
    type Error = Error;
    fn erase(&mut self) -> nb::Result<(), Self::Error> {
        // Early yield if flash is not ready for writing
        if Self::status(&mut self.qspi)?.write_in_progress {
            Err(nb::Error::WouldBlock)
        } else {
            Self::execute_command(&mut self.qspi, Command::WriteEnable, None, CommandData::None)?;
            Self::execute_command(&mut self.qspi, Command::BulkErase, None, CommandData::None)?;
            Self::execute_command(&mut self.qspi, Command::WriteDisable, None, CommandData::None)?;
            Ok(())
        }
    }
}

impl<QSPI, NOW, I> Write<Address> for MicronN25q128a<QSPI, NOW, I>
where
    QSPI: qspi::Indirect,
    NOW: time::Now<I>,
    I: time::Instant,
{
    type Error = Error;

    fn write(&mut self, address: Address, bytes: &[u8]) -> nb::Result<(), Self::Error> {
        // TODO remove page alignment limitations
        if (address.0 % 256 != 0) || bytes.len() > 256 {
            return Err(nb::Error::Other(Error::MisalignedAccess));
        }

        // TODO read subsector first before erasing (to preserve previous values)
        block!(Self::execute_command(
            &mut self.qspi,
            Command::WriteEnable,
            None,
            CommandData::None
        ))?;
        block!(Self::execute_command(
            &mut self.qspi,
            Command::SubsectorErase,
            Some(address),
            CommandData::None
        ))?;
        block!(self.wait_until_write_complete())?;
        block!(Self::execute_command(
            &mut self.qspi,
            Command::WriteEnable,
            None,
            CommandData::None
        ))?;
        block!(Self::execute_command(
            &mut self.qspi,
            Command::PageProgram,
            Some(address),
            CommandData::Write(&bytes)
        ))?;
        Ok(())
    }

    fn writable_range() -> (Address, Address) {
        unimplemented!();
    }
}

impl<QSPI, NOW, I> Read<Address> for MicronN25q128a<QSPI, NOW, I>
where
    QSPI: qspi::Indirect,
    NOW: time::Now<I>,
    I: time::Instant,
{
    type Error = Error;
    fn read(&mut self, address: Address, bytes: &mut [u8]) -> nb::Result<(), Self::Error> {
        if Self::status(&mut self.qspi)?.write_in_progress {
            Err(nb::Error::WouldBlock)
        } else {
            Self::execute_command(
                &mut self.qspi,
                Command::Read,
                Some(address),
                CommandData::Read(bytes),
            )
        }
    }

    fn readable_range() -> (Address, Address) {
        unimplemented!();
    }
}

impl<QSPI, NOW, I> MicronN25q128a<QSPI, NOW, I>
where
    QSPI: qspi::Indirect,
    NOW: time::Now<I>,
    I: time::Instant,
{
    fn wait_until_write_complete(&mut self) -> nb::Result<(), Error> {
        if let Some((timeout, systick)) = &self.timeout {
            let start = systick.now();
            while Self::status(&mut self.qspi)?.write_in_progress {
                if systick.now() - start > *timeout {
                    return Err(nb::Error::Other(Error::TimeOut));
                }
            }
        }

        if Self::status(&mut self.qspi)?.write_in_progress {
            Err(nb::Error::WouldBlock)
        } else {
            Ok(())
        }
    }

    // Low level helper for executing Micron commands
    fn execute_command(
        qspi: &mut QSPI,
        command: Command,
        address: Option<Address>,
        data: CommandData,
    ) -> nb::Result<(), Error> {
        match data {
            CommandData::Write(buffer) => {
                block!(qspi.write(Some(command as u8), address.map(|a| a.0), Some(buffer), 0))
            }
            CommandData::Read(buffer) => {
                block!(qspi.read(Some(command as u8), address.map(|a| a.0), buffer, 0))
            }
            CommandData::None => {
                block!(qspi.write(Some(command as u8), address.map(|a| a.0), None, 0))
            }
        }
        .map_err(|_| nb::Error::Other(Error::QspiError))
    }

    fn verify_id(&mut self) -> nb::Result<(), Error> {
        let mut response = [0u8; 1];
        Self::execute_command(
            &mut self.qspi,
            Command::ReadId,
            None,
            CommandData::Read(&mut response),
        )?;
        match response[0] {
            MANUFACTURER_ID => Ok(()),
            _ => Err(nb::Error::Other(Error::WrongManufacturerId)),
        }
    }

    fn status(qspi: &mut QSPI) -> nb::Result<Status, Error> {
        let mut response = [0u8; 1];
        Self::execute_command(qspi, Command::ReadStatus, None, CommandData::Read(&mut response))?;
        let response = response[0];
        Ok(Status {
            write_in_progress: response.is_set(0),
            _write_enable_latch: response.is_set(1),
        })
    }

    /// Blocks until flash ID read checks out, or until timeout
    pub fn new(qspi: QSPI) -> Result<Self, Error> {
        let mut flash = Self { qspi, timeout: None, _marker: PhantomData::default() };
        block!(flash.verify_id())?;
        Ok(flash)
    }

    pub fn with_timeout(
        qspi: QSPI,
        timeout: time::Milliseconds,
        systick: NOW,
    ) -> Result<Self, Error> {
        let mut flash =
            Self { qspi, timeout: Some((timeout, systick)), _marker: PhantomData::default() };
        block!(flash.verify_id())?;
        Ok(flash)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::hal::doubles::{gpio::*, qspi::*, time::*};

    type FlashToTest = MicronN25q128a<MockQspi, MockSysTick, MockInstant>;
    fn flash_to_test() -> FlashToTest {
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
        assert!(FlashToTest::new(qspi).is_err());

        // Given
        let mut qspi = MockQspi::default();
        qspi.to_read.push_back(vec![MANUFACTURER_ID]);

        // Then
        assert!(FlashToTest::new(qspi).is_ok());
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
