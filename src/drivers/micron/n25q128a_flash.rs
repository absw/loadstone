//! Device driver for the [Micron N24q128a](../../../../../../../../documentation/hardware/micron_flash.pdf#page=0)
use crate::{
    hal::{
        flash::{BulkErase, Read, Write},
        qspi, time,
    },
    static_assertions::const_assert,
    utilities::{
        bitwise::BitFlags,
        memory::{self, IterableByBlocksAndRegions, Region},
    },
};
use core::ops::Add;
use nb::block;

/// From [datasheet table 19](../../../../../../../../documentation/hardware/micron_flash.pdf#page=37)
const MANUFACTURER_ID: u8 = 0x20;

/// Address into the micron chip [memory map](../../../../../../../../documentation/hardware/micron_flash.pdf#page=14)
#[derive(Default, Copy, Clone, Debug, PartialOrd, PartialEq)]
pub struct Address(u32);
impl Add<usize> for Address {
    type Output = Self;
    fn add(self, rhs: usize) -> Address { Address(self.0 + rhs as u32) }
}

struct MemoryMap {}
struct Sector(usize);
struct Subsector(usize);
struct Page(usize);

// Existential iterator types (alias for `some` type that iterates over them)
type Sectors = impl Iterator<Item = Sector>;
type Subsectors = impl Iterator<Item = Subsector>;
type Pages = impl Iterator<Item = Page>;

impl MemoryMap {
    fn sectors() -> Sectors { (0..NUMBER_OF_SECTORS).map(Sector) }
    fn subsectors() -> Subsectors { (0..NUMBER_OF_SUBSECTORS).map(Subsector) }
    fn pages() -> Pages { (0..NUMBER_OF_PAGES).map(Page) }
    const fn location() -> Address { BASE_ADDRESS }
    const fn end() -> Address { Address(BASE_ADDRESS.0 + MEMORY_SIZE as u32) }
    const fn size() -> usize { MEMORY_SIZE }
}

impl Sector {
    fn subsectors(&self) -> Subsectors {
        ((self.0 * SUBSECTORS_PER_SECTOR)..((1 + self.0) * SUBSECTORS_PER_SECTOR)).map(Subsector)
    }
    fn pages(&self) -> Pages { (self.0..(self.0 + PAGES_PER_SECTOR)).map(Page) }
    fn location(&self) -> Address { BASE_ADDRESS + self.0 * Self::size() }
    fn end(&self) -> Address { self.location() + Self::size() }
    const fn size() -> usize { SECTOR_SIZE }
}

impl Subsector {
    fn pages(&self) -> Pages {
        ((self.0 * PAGES_PER_SUBSECTOR)..((1 + self.0) * PAGES_PER_SUBSECTOR)).map(Page)
    }
    fn location(&self) -> Address { BASE_ADDRESS + self.0 * Self::size() }
    fn end(&self) -> Address { self.location() + Self::size() }
    const fn size() -> usize { SUBSECTOR_SIZE }
}

impl Page {
    fn location(&self) -> Address { BASE_ADDRESS + self.0 * Self::size() }
    fn end(&self) -> Address { self.location() + Self::size() }
    const fn size() -> usize { PAGE_SIZE }
}

impl memory::Region<Address> for MemoryMap {
    fn contains(&self, address: Address) -> bool {
        (address >= BASE_ADDRESS) && (address < BASE_ADDRESS + MEMORY_SIZE)
    }
}

impl memory::Region<Address> for Sector {
    fn contains(&self, address: Address) -> bool {
        let start = Address((Self::size() * self.0) as u32);
        (address >= start) && (address < start + Self::size())
    }
}

impl memory::Region<Address> for Subsector {
    fn contains(&self, address: Address) -> bool {
        let start = Address((Self::size() * self.0) as u32);
        (address >= start) && (address < start + Self::size())
    }
}

impl memory::Region<Address> for Page {
    fn contains(&self, address: Address) -> bool {
        let start = Address((Self::size() * self.0) as u32);
        (address >= start) && (address < start + Self::size())
    }
}

const BASE_ADDRESS: Address = Address(0x0000_0000);

const PAGES_PER_SUBSECTOR: usize = 16;
const SUBSECTORS_PER_SECTOR: usize = 16;
const PAGES_PER_SECTOR: usize = PAGES_PER_SUBSECTOR * SUBSECTORS_PER_SECTOR;

const PAGE_SIZE: usize = 256;
const SUBSECTOR_SIZE: usize = PAGE_SIZE * PAGES_PER_SUBSECTOR;
const SECTOR_SIZE: usize = SUBSECTOR_SIZE * SUBSECTORS_PER_SECTOR;
const MEMORY_SIZE: usize = NUMBER_OF_SECTORS * SECTOR_SIZE;

const NUMBER_OF_SECTORS: usize = 256;
const NUMBER_OF_SUBSECTORS: usize = NUMBER_OF_SECTORS * SUBSECTORS_PER_SECTOR;
const NUMBER_OF_PAGES: usize = NUMBER_OF_SUBSECTORS * PAGES_PER_SUBSECTOR;

/// MicronN25q128a driver, generic over a QSPI programmed in indirect mode
pub struct MicronN25q128a<QSPI, NOW>
where
    QSPI: qspi::Indirect,
    NOW: time::Now,
{
    qspi: QSPI,
    timeout: Option<(time::Milliseconds, NOW)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    TimeOut,
    QspiError,
    WrongManufacturerId,
    MisalignedAccess,
    AddressOutOfRange,
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

impl<QSPI, NOW> BulkErase for MicronN25q128a<QSPI, NOW>
where
    QSPI: qspi::Indirect,
    NOW: time::Now,
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

impl<QSPI, NOW> Write for MicronN25q128a<QSPI, NOW>
where
    QSPI: qspi::Indirect,
    NOW: time::Now,
{
    type Error = Error;
    type Address = Address;

    fn write(&mut self, address: Address, bytes: &[u8]) -> nb::Result<(), Self::Error> {
        //block!(self.erase_subsector(Subsector::at(&address)?))?;
        unimplemented!("requires page writes (lower granularity than sectors)");
        //block!(Self::execute_command(
        //    &mut self.qspi,
        //    Command::WriteEnable,
        //    None,
        //    CommandData::None
        //))?;
        //block!(Self::execute_command(
        //    &mut self.qspi,
        //    Command::PageProgram,
        //    Some(address),
        //    CommandData::Write(&bytes)
        //))?;
        //Ok(())
    }

    fn writable_range() -> (Address, Address) {
        // TODO write a proper table instead of hardcoding it
        (Address(0x0000_0000), Address(0x00FF_0000))
    }
}

impl<QSPI, NOW> Read for MicronN25q128a<QSPI, NOW>
where
    QSPI: qspi::Indirect,
    NOW: time::Now,
{
    type Error = Error;
    type Address = Address;
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
        // TODO write a proper table instead of hardcoding it
        (Address(0x0000_0000), Address(0x00FF_0000))
    }
}

impl<QSPI, NOW> MicronN25q128a<QSPI, NOW>
where
    QSPI: qspi::Indirect,
    NOW: time::Now,
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
        let mut flash = Self { qspi, timeout: None };
        block!(flash.verify_id())?;
        Ok(flash)
    }

    pub fn with_timeout(
        qspi: QSPI,
        timeout: time::Milliseconds,
        systick: NOW,
    ) -> Result<Self, Error> {
        let mut flash = Self { qspi, timeout: Some((timeout, systick)) };
        block!(flash.verify_id())?;
        Ok(flash)
    }

    fn erase_subsector(&mut self, subsector: &Subsector) -> nb::Result<(), Error> {
        block!(Self::execute_command(
            &mut self.qspi,
            Command::WriteEnable,
            None,
            CommandData::None
        ))?;
        block!(Self::execute_command(
            &mut self.qspi,
            Command::SubsectorErase,
            Some(subsector.location()),
            CommandData::None
        ))?;
        Ok(block!(self.wait_until_write_complete())?)
    }

    fn write_page(&mut self, page: &Page, bytes: &[u8], address: Address) -> nb::Result<(), Error> {
        if (address < page.location()) || (address + bytes.len() > page.end()) {
            return Err(nb::Error::Other(Error::MisalignedAccess));
        }

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
        Ok(block!(self.wait_until_write_complete())?)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::hal::doubles::{gpio::*, qspi::*, time::*};

    type FlashToTest = MicronN25q128a<MockQspi, MockSysTick>;
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
    fn various_memory_map_iterations() {
        assert_eq!(MemoryMap::sectors().count(), NUMBER_OF_SECTORS);
        assert_eq!(MemoryMap::subsectors().count(), NUMBER_OF_SUBSECTORS);
        assert_eq!(MemoryMap::pages().count(), NUMBER_OF_PAGES);

        let expected_address = Address((3 * SECTOR_SIZE + 3 * SUBSECTOR_SIZE) as u32);
        let expected_index = 3 * SUBSECTORS_PER_SECTOR + 3;
        let subsector = MemoryMap::sectors().nth(3).unwrap().subsectors().nth(3).unwrap();
        assert_eq!(expected_address, subsector.location());
        assert_eq!(subsector.0, expected_index);

        let expected_address = Address((1 * SECTOR_SIZE + 2 * SUBSECTOR_SIZE + 3 * PAGE_SIZE) as u32);
        let expected_index = 1 * PAGES_PER_SECTOR + 2 * PAGES_PER_SUBSECTOR + 3;
        let page = MemoryMap::sectors().nth(1).unwrap().subsectors().nth(2).unwrap().pages().nth(3).unwrap();
        assert_eq!(expected_address, page.location());
        assert_eq!(page.0, expected_index);
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
