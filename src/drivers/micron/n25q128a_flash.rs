//! Device driver for the [Micron N24q128a](../../../../../../documentation/hardware/micron_flash.pdf#page=0)
use crate::{
    hal::{
        flash::{BulkErase, ReadWrite},
        qspi, time,
    },
    utilities::{
        bitwise::{BitFlags, SliceBitSubset},
        memory::{self, IterableByOverlaps, Region},
    },
};
use core::ops::{Add, Sub};
use nb::block;

/// From [datasheet table 19](../../../../../../../documentation/hardware/micron_flash.pdf#page=37)
pub const MANUFACTURER_ID: u8 = 0x20;

/// Address into the micron chip [memory map](../../../../../../../documentation/hardware/micron_flash.pdf#page=14)
#[derive(Default, Copy, Clone, Debug, PartialOrd, PartialEq, Eq, Ord)]
pub struct Address(pub u32);
impl Add<usize> for Address {
    type Output = Self;
    fn add(self, rhs: usize) -> Address { Address(self.0 + rhs as u32) }
}
impl Sub<usize> for Address {
    type Output = Self;
    fn sub(self, rhs: usize) -> Address { Address(self.0.saturating_sub(rhs as u32)) }
}
impl Sub<Address> for Address {
    type Output = usize;
    fn sub(self, rhs: Address) -> usize { self.0.saturating_sub(rhs.0) as usize }
}
impl Into<usize> for Address {
    fn into(self) -> usize { self.0 as usize }
}

pub struct MemoryMap {}
pub struct Sector(usize);
pub struct Subsector(usize);
pub struct Page(usize);

impl MemoryMap {
    pub fn sectors() -> impl Iterator<Item = Sector> { (0..NUMBER_OF_SECTORS).map(Sector) }
    pub fn subsectors() -> impl Iterator<Item = Subsector> {
        (0..NUMBER_OF_SUBSECTORS).map(Subsector)
    }
    pub fn pages() -> impl Iterator<Item = Page> { (0..NUMBER_OF_PAGES).map(Page) }
    pub const fn location() -> Address { BASE_ADDRESS }
    pub const fn end() -> Address { Address(BASE_ADDRESS.0 + MEMORY_SIZE as u32) }
    pub const fn size() -> usize { MEMORY_SIZE }
}

impl Sector {
    pub fn subsectors(&self) -> impl Iterator<Item = Subsector> {
        ((self.0 * SUBSECTORS_PER_SECTOR)..((1 + self.0) * SUBSECTORS_PER_SECTOR)).map(Subsector)
    }
    pub fn pages(&self) -> impl Iterator<Item = Page> {
        ((self.0 * PAGES_PER_SECTOR)..((1 + self.0) * PAGES_PER_SECTOR)).map(Page)
    }
    pub fn location(&self) -> Address { BASE_ADDRESS + self.0 * Self::size() }
    pub fn end(&self) -> Address { self.location() + Self::size() }
    pub fn at(address: Address) -> Option<Self> {
        MemoryMap::sectors().find(|s| s.contains(address))
    }
    pub const fn size() -> usize { SECTOR_SIZE }
}

impl Subsector {
    pub fn pages(&self) -> impl Iterator<Item = Page> {
        ((self.0 * PAGES_PER_SUBSECTOR)..((1 + self.0) * PAGES_PER_SUBSECTOR)).map(Page)
    }
    pub fn location(&self) -> Address { BASE_ADDRESS + self.0 * Self::size() }
    pub fn end(&self) -> Address { self.location() + Self::size() }
    pub fn at(address: Address) -> Option<Self> {
        MemoryMap::subsectors().find(|s| s.contains(address))
    }
    pub const fn size() -> usize { SUBSECTOR_SIZE }
}

impl Page {
    pub fn location(&self) -> Address { BASE_ADDRESS + self.0 * Self::size() }
    pub fn end(&self) -> Address { self.location() + Self::size() }
    pub fn at(address: Address) -> Option<Self> { MemoryMap::pages().find(|p| p.contains(address)) }
    pub const fn size() -> usize { PAGE_SIZE }
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

impl<QSPI, NOW> ReadWrite for MicronN25q128a<QSPI, NOW>
where
    QSPI: qspi::Indirect,
    NOW: time::Now,
{
    type Error = Error;
    type Address = Address;

    fn write(&mut self, address: Address, bytes: &[u8]) -> nb::Result<(), Self::Error> {
        if Self::status(&mut self.qspi)?.write_in_progress {
            return Err(nb::Error::WouldBlock);
        }

        for (bytes, subsector, address) in MemoryMap::subsectors().overlaps(bytes, address) {
            let offset_into_subsector = address - subsector.location();
            let mut subsector_data = [0x00u8; SUBSECTOR_SIZE];
            block!(self.read(subsector.location(), &mut subsector_data))?;
            if bytes.is_subset_of(&mut subsector_data[offset_into_subsector..]) {
                for (bytes, page, address) in subsector.pages().overlaps(bytes, address) {
                    block!(self.write_page(&page, bytes, address))?;
                }
            } else {
                block!(self.erase_subsector(&subsector))?;
                // "merge" the preexisting data with the new write.
                subsector_data
                    .iter_mut()
                    .skip(offset_into_subsector)
                    .zip(bytes)
                    .for_each(|(a, b)| *a = *b);
                for (bytes, page, address) in
                    subsector.pages().overlaps(&subsector_data, subsector.location())
                {
                    block!(self.write_page(&page, bytes, address))?;
                }
            }
        }
        Ok(())
    }

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

    fn range() -> (Address, Address) { (MemoryMap::location(), MemoryMap::end()) }
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
        if Self::status(&mut self.qspi)?.write_in_progress {
            return Err(nb::Error::WouldBlock);
        }
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
        if Self::status(&mut self.qspi)?.write_in_progress {
            return Err(nb::Error::WouldBlock);
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
    use std::collections::VecDeque;

    const NOT_BUSY: u8 = 0x0u8;
    const COMMANDS_PER_PAGE_WRITE: usize = 4;

    type FlashToTest = MicronN25q128a<MockQspi, MockSysTick>;
    fn flash_to_test() -> FlashToTest {
        let mut qspi = MockQspi::default();
        qspi.to_read.push_back(vec![MANUFACTURER_ID]);
        let mut flash = MicronN25q128a::new(qspi).unwrap();
        let initial_read = flash.qspi.command_records[0].clone();
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

        let expected_address =
            Address((1 * SECTOR_SIZE + 2 * SUBSECTOR_SIZE + 3 * PAGE_SIZE) as u32);
        let expected_index = 1 * PAGES_PER_SECTOR + 2 * PAGES_PER_SUBSECTOR + 3;
        let page = MemoryMap::sectors()
            .nth(1)
            .unwrap()
            .subsectors()
            .nth(2)
            .unwrap()
            .pages()
            .nth(3)
            .unwrap();
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
        let records = &flash.qspi.command_records;

        // Then
        assert_eq!(records[0].instruction, Some(Command::ReadStatus as u8));
        assert_eq!(records[1].instruction, Some(Command::WriteEnable as u8));
        assert_eq!(records[2].instruction, Some(Command::BulkErase as u8));
        assert_eq!(records[3].instruction, Some(Command::WriteDisable as u8));
    }

    #[test]
    fn write_capable_commands_yield_if_device_busy() {
        // Given
        const BUSY_WRITING_STATUS: u8 = 1;
        let mut flash = flash_to_test();
        flash.qspi.to_read.push_back(vec![BUSY_WRITING_STATUS]);

        // Then
        assert_eq!(flash.erase(), Err(nb::Error::WouldBlock));

        flash.qspi.to_read.push_back(vec![BUSY_WRITING_STATUS]);
    }

    #[test]
    fn page_program_command_sequence() {
        // Given
        let mut flash = flash_to_test();
        let address = Address(0x1000);
        let data = [0xAAu8; PAGE_SIZE];

        // When
        flash.write_page(&Page::at(address).unwrap(), &data, address).unwrap();
        let records = &flash.qspi.command_records;

        // Then
        assert_eq!(records[0].instruction, Some(Command::ReadStatus as u8));
        assert_eq!(records[1].instruction, Some(Command::WriteEnable as u8));
        assert_eq!(records[2].instruction, Some(Command::PageProgram as u8));
        assert_eq!(Some(address.0), records[2].address);
        assert!(records[2].contains(&data));
    }

    #[test]
    fn subsector_read_command_sequence() {
        // Given
        let mut flash = flash_to_test();
        let address = MemoryMap::subsectors().nth(12).unwrap().location();
        let mut data = [0x00u8; SUBSECTOR_SIZE];

        // When
        flash.read(address, &mut data).unwrap();
        let records = &flash.qspi.command_records;

        // Then
        assert_eq!(records[0].instruction, Some(Command::ReadStatus as u8));
        assert_eq!(records[1].instruction, Some(Command::Read as u8));
        assert_eq!(Some(address.0), records[1].address);
        assert_eq!(SUBSECTOR_SIZE, records[1].length_requested);
    }

    #[test]
    fn writing_a_bitwise_subset_of_a_subsector() {
        // Given
        let mut flash = flash_to_test();
        let data_to_write = [0xAA, 0xBB, 0xAA, 0xBB];
        let subsector = MemoryMap::subsectors().nth(12).unwrap();
        let page = subsector.pages().nth(3).unwrap();

        flash.qspi.to_read = VecDeque::from(vec![
            vec![NOT_BUSY],             // Response to busy check when calling write
            vec![NOT_BUSY],             // Response to busy check when calling first read
            vec![0xFF; SUBSECTOR_SIZE], //sector data (for pre-write check)
        ]);

        // When
        flash.write(page.location(), &data_to_write).unwrap();
        let records = &flash.qspi.command_records;

        // Then we read the sector to verify we are a subset
        assert_eq!(records[0].instruction, Some(Command::ReadStatus as u8));
        assert_eq!(records[1].instruction, Some(Command::ReadStatus as u8));
        assert_eq!(records[2].instruction, Some(Command::Read as u8));
        assert_eq!(Some(subsector.location().0), records[2].address);
        assert_eq!(SUBSECTOR_SIZE, records[2].length_requested);
        records[1].contains(&[0xFF; SUBSECTOR_SIZE]);

        // And we are a subset, so we simply write the data
        assert_eq!(records[3].instruction, Some(Command::ReadStatus as u8));
        assert_eq!(records[4].instruction, Some(Command::WriteEnable as u8));
        assert_eq!(records[5].instruction, Some(Command::PageProgram as u8));
        assert_eq!(Some(page.location().0), records[5].address);
        assert!(records[5].contains(&data_to_write));
    }

    fn wrote_a_whole_subsector(data: &[u8], address: Address, commands: &[CommandRecord]) -> bool {
        (0..PAGES_PER_SUBSECTOR).map(|i| (i, i * COMMANDS_PER_PAGE_WRITE)).all(|(page, i)| {
            commands[i].instruction == Some(Command::ReadStatus as u8)
                && commands[i + 1].instruction == Some(Command::WriteEnable as u8)
                && commands[i + 2].instruction == Some(Command::PageProgram as u8)
                && commands[i + 2].address == Some((address + page * PAGE_SIZE).0)
                && commands[i + 2].contains(&data[page * PAGE_SIZE..(page + 1) * PAGE_SIZE])
                && commands[i + 3].instruction == Some(Command::ReadStatus as u8)
        })
    }

    #[test]
    fn writing_a_whole_subsector_page_by_page() {
        // Given
        let mut flash = flash_to_test();
        let data_to_write = [0x00; SUBSECTOR_SIZE];
        let subsector = MemoryMap::subsectors().nth(12).unwrap();

        // When
        flash.write(subsector.location(), &data_to_write).unwrap();
        let records = &flash.qspi.command_records;

        // Then we read the subsector to verify we are a subset of it
        assert_eq!(records[0].instruction, Some(Command::ReadStatus as u8));
        assert_eq!(records[1].instruction, Some(Command::ReadStatus as u8));
        assert_eq!(records[2].instruction, Some(Command::Read as u8));

        // And we write the whole thing page by page
        assert!(wrote_a_whole_subsector(&data_to_write, subsector.location(), &records[3..]));
    }

    #[test]
    fn writing_a_non_bitwise_subset_of_a_subsector() {
        // Given
        let mut flash = flash_to_test();
        let data_to_write = [0xAA, 0xBB, 0xAA, 0xBB];
        let subsector = MemoryMap::subsectors().nth(12).unwrap();
        let page = subsector.pages().nth(3).unwrap();
        let original_subsector_data = vec![0x11u8; SUBSECTOR_SIZE];
        let mut merged_data = original_subsector_data.clone();
        merged_data
            .iter_mut()
            .skip(page.location() - subsector.location())
            .zip(data_to_write.iter())
            .for_each(|(a, b)| *a = *b);

        flash.qspi.to_read = VecDeque::from(vec![
            vec![NOT_BUSY],          // Response to busy check when calling write
            vec![NOT_BUSY],          // Response to busy check when calling first read
            original_subsector_data, //sector data (for pre-write check). Not a superset!
        ]);

        // When
        flash.write(page.location(), &data_to_write).unwrap();
        let records = &flash.qspi.command_records;
        assert_eq!(records[0].instruction, Some(Command::ReadStatus as u8));

        // Then we read the sector to verify we are a subset
        assert_eq!(records[1].instruction, Some(Command::ReadStatus as u8));
        assert_eq!(records[2].instruction, Some(Command::Read as u8));
        assert_eq!(Some(subsector.location().0), records[2].address);
        assert_eq!(SUBSECTOR_SIZE, records[2].length_requested);
        records[1].contains(&[0x11; SUBSECTOR_SIZE]);

        // And we are not a subset, so we erase first
        assert_eq!(records[3].instruction, Some(Command::ReadStatus as u8));
        assert_eq!(records[4].instruction, Some(Command::WriteEnable as u8));
        assert_eq!(records[5].instruction, Some(Command::SubsectorErase as u8));
        assert_eq!(Some(subsector.location().0), records[5].address);
        assert_eq!(records[6].instruction, Some(Command::ReadStatus as u8));

        // And then we write the "merged" data back, page per page
        assert!(wrote_a_whole_subsector(&merged_data, subsector.location(), &records[7..]));
    }

    #[test]
    fn write_straddling_two_subsectors() {
        // Given
        let mut flash = flash_to_test();
        let data_to_write = [0x00u8; 2 * SUBSECTOR_SIZE];
        let address = MemoryMap::subsectors().nth(1).unwrap().location();

        // When
        flash.write(address, &data_to_write).unwrap();
        let records = &flash.qspi.command_records;

        // Then
        // First subsector subset check
        assert_eq!(records[0].instruction, Some(Command::ReadStatus as u8));
        assert_eq!(records[1].instruction, Some(Command::ReadStatus as u8));
        assert_eq!(records[2].instruction, Some(Command::Read as u8));

        // And first subsector write
        assert!(wrote_a_whole_subsector(&data_to_write[..SUBSECTOR_SIZE], address, &records[3..]));

        let index = 3 + COMMANDS_PER_PAGE_WRITE * PAGES_PER_SUBSECTOR;

        // second subsector subset check
        assert_eq!(records[index].instruction, Some(Command::ReadStatus as u8));
        assert_eq!(records[index + 1].instruction, Some(Command::Read as u8));

        // And second subsector write
        assert!(wrote_a_whole_subsector(
            &data_to_write[SUBSECTOR_SIZE..],
            address + SUBSECTOR_SIZE,
            &records[index + 2..]
        ));
    }
}
