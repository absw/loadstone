//! Internal Flash controller for the STM32F4 family

use crate::{
    error::Error as BootloaderError,
    hal::flash::{Read, Write},
    stm32pac::FLASH,
};
use core::ops::Add;
use nb::block;

pub struct McuFlash {
    flash: FLASH,
}

#[derive(Copy, Clone, Debug)]
pub enum Error {
    MemoryNotReachable,
    MisalignedAccess,
}

impl From<Error> for BootloaderError {
    fn from(error: Error) -> Self {
        BootloaderError::DriverError(match error {
            Error::MemoryNotReachable => "MCU flash memory not reachable",
            Error::MisalignedAccess => "MCU flash memory access misaligned",
        })
    }
}

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq)]
pub struct Address(u32);
impl Add<u32> for Address {
    type Output = Self;
    fn add(self, rhs: u32) -> Address { Address(self.0 + rhs) }
}

#[derive(Copy, Clone, Debug)]
struct Range(Address, Address);

/// Different address blocks as defined in [Table 5](../../../../../../../../documentation/hardware/stm32f412_reference.pdf#page=58)
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Block {
    /// Main memory, but reserved for secure bootloader
    Boot,
    /// Main memory, where the application is written
    Main,
    SystemMemory,
    OneTimeProgrammable,
    OptionBytes,
}

/// A memory map sector, with an associated block and an address range
#[derive(Copy, Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct Sector {
    pub block: Block,
    pub start: Address,
    pub size: u32,
}

#[non_exhaustive]
pub struct MemoryMap {
    pub sectors: [Sector; SECTOR_NUMBER],
}

///From [section 3.5.1](../../../../../../../../documentation/hardware/stm32f412_reference.pdf#page=62)
const UNLOCK_KEYS: [u32; 2] = [0x45670123, 0xCDEF89AB];

#[cfg(feature = "stm32f412")]
pub const SECTOR_NUMBER: usize = 15;

#[cfg(feature = "stm32f412")]
pub const MEMORY_MAP: MemoryMap = MemoryMap {
    sectors: [
        Sector::new(Block::Boot, 0x0800_0000, 0x4000),
        Sector::new(Block::Boot, 0x0800_4000, 0x4000),
        Sector::new(Block::Boot, 0x0800_8000, 0x4000),
        Sector::new(Block::Boot, 0x0800_C000, 0x4000),
        Sector::new(Block::Main, 0x0801_0000, 0x10000),
        Sector::new(Block::Main, 0x0802_0000, 0x20000),
        Sector::new(Block::Main, 0x0804_0000, 0x20000),
        Sector::new(Block::Main, 0x0806_0000, 0x20000),
        Sector::new(Block::Main, 0x0808_0000, 0x20000),
        Sector::new(Block::Main, 0x080A_0000, 0x20000),
        Sector::new(Block::Main, 0x080C_0000, 0x20000),
        Sector::new(Block::Main, 0x080E_0000, 0x20000),
        Sector::new(Block::SystemMemory, 0x1FFF_0000, 0x7800),
        Sector::new(Block::OneTimeProgrammable, 0x1FFF_7800, 0x210),
        Sector::new(Block::OptionBytes, 0x1FFF_C000, 0x10),
    ],
};

impl MemoryMap {
    // Verifies that the memory map is consecutive and well formed
    fn is_sound(&self) -> bool {
        let main_sectors = self.sectors.iter().filter(|s| s.is_in_main_memory_area());
        let mut consecutive_pairs = main_sectors.clone().zip(main_sectors.skip(1));
        let consecutive = consecutive_pairs.all(|(a, b)| a.start + a.size == b.start);
        let ranges_valid =
            self.sectors.iter().map(|s| Range(s.start, s.start + s.size)).all(Range::is_valid);
        consecutive && ranges_valid
    }
}

impl Range {
    /// Sectors spanned by this range of addresses
    fn span(self) -> &'static [Sector] {
        let first = MEMORY_MAP
            .sectors
            .iter()
            .enumerate()
            .find_map(|(i, sector)| self.overlaps(sector).then_some(i));
        let last = MEMORY_MAP
            .sectors
            .iter()
            .enumerate()
            .rev()
            .find_map(|(i, sector)| self.overlaps(sector).then_some(i));
        match (first, last) {
            (Some(first), Some(last)) if (last >= first) => &MEMORY_MAP.sectors[first..(last + 1)],
            _ => &MEMORY_MAP.sectors[0..1],
        }
    }

    const fn is_valid(self) -> bool {
        let Range(Address(start), Address(end)) = self;
        let after_map = start
            >= (MEMORY_MAP.sectors[SECTOR_NUMBER - 1].start.0
                + MEMORY_MAP.sectors[SECTOR_NUMBER - 1].size as u32);
        let before_map = end < MEMORY_MAP.sectors[0].start.0;
        let monotonic = end >= start;
        monotonic && !before_map && !after_map
    }

    fn overlaps(self, sector: &Sector) -> bool {
        (self.0 <= sector.start) && (self.1 > sector.start)
            || (self.0 < (sector.start + sector.size)) && (self.1 > (sector.start + sector.size))
    }

    /// Verify that all sectors spanned by this range are writable
    fn is_writable(self) -> bool { self.span().iter().all(Sector::is_writable) }
}

impl Sector {
    const fn new(block: Block, start: u32, size: u32) -> Self {
        Sector { block, start: Address(start), size }
    }
    fn number(&self) -> Option<u8> {
        MEMORY_MAP.sectors.iter().enumerate().find_map(|(index, sector)| {
            (sector.is_in_main_memory_area() && self == sector).then_some(index as u8)
        })
    }
    const fn is_writable(&self) -> bool { self.block as u8 == Block::Main as u8 }
    const fn is_in_main_memory_area(&self) -> bool {
        self.block as u8 == Block::Main as u8 || self.block as u8 == Block::Boot as u8
    }
}

impl McuFlash {
    pub fn new(flash: FLASH) -> Result<Self, Error> {
        assert!(MEMORY_MAP.is_sound());
        Ok(Self { flash })
    }

    /// Parallelism for 3v3 voltage from [table 7](../../../../../../../../documentation/hardware/stm32f412_reference.pdf#page=63)
    /// (Word access parallelism)
    fn unlock(&mut self) -> nb::Result<(), Error> {
        if self.is_busy() {
            return Err(nb::Error::WouldBlock);
        }
        // NOTE(Safety): Unsafe block to use the 'bits' convenience function.
        // Applies to all blocks in this file unless specified otherwise
        self.flash.keyr.write(|w| unsafe { w.bits(UNLOCK_KEYS[0]) });
        self.flash.keyr.write(|w| unsafe { w.bits(UNLOCK_KEYS[1]) });
        self.flash.cr.modify(|_, w| unsafe { w.psize().bits(0b10) });
        Ok(())
    }

    fn lock(&mut self) { self.flash.cr.modify(|_, w| w.lock().set_bit()); }

    fn erase(&mut self, sector: &Sector) -> nb::Result<(), Error> {
        let number = sector.number().ok_or(nb::Error::Other(Error::MemoryNotReachable))?;
        self.unlock()?;
        self.flash
            .cr
            .modify(|_, w| unsafe { w.ser().set_bit().snb().bits(number).strt().set_bit() });
        self.lock();
        Ok(())
    }

    fn is_busy(&self) -> bool { self.flash.sr.read().bsy().bit_is_set() }
}

impl Write for McuFlash {
    type Error = Error;
    type Address = Address;

    fn writable_range() -> (Address, Address) {
        let mut writable_sectors = MEMORY_MAP.sectors.iter().filter(|s| s.is_writable());
        let (first, last) = (writable_sectors.next().unwrap(), writable_sectors.last().unwrap());
        let range = Range(first.start, last.start + last.size);
        (range.0, range.1)
    }

    fn write(&mut self, address: Address, bytes: &[u8]) -> nb::Result<(), Self::Error> {
        if address.0 % 4 != 0 {
            return Err(nb::Error::Other(Error::MisalignedAccess));
        }

        // Adjust end for alignment
        let range = Range(address, Address(address.0 + bytes.len() as u32));
        if !range.is_writable() {
            return Err(nb::Error::Other(Error::MemoryNotReachable));
        }

        // Early yield if busy
        if self.is_busy() {
            return Err(nb::Error::WouldBlock);
        }

        //TODO smart read-write cycle
        for sector in range.span() {
            block!(self.erase(sector))?;
        }

        let words = bytes.chunks(4).map(|bytes| {
            u32::from_le_bytes([
                bytes.get(0).cloned().unwrap_or(0),
                bytes.get(1).cloned().unwrap_or(0),
                bytes.get(2).cloned().unwrap_or(0),
                bytes.get(3).cloned().unwrap_or(0),
            ])
        });
        block!(self.unlock())?;
        self.flash.cr.modify(|_, w| w.pg().set_bit());
        let base_address = address.0 as *mut u32;
        for (index, word) in words.enumerate() {
            // NOTE(Safety): Writing to a memory-mapped flash
            // directly is naturally unsafe. We have to trust that
            // the memory map is correct, and that these dereferences
            // won't cause a hardfault or overlap with our firmware.
            unsafe {
                *(base_address.add(index)) = word;
            }
        }
        self.lock();
        Ok(())
    }
}

impl Read for McuFlash {
    type Error = Error;
    type Address = Address;

    fn readable_range() -> (Address, Address) { Self::writable_range() }
    fn read(&mut self, address: Address, bytes: &mut [u8]) -> nb::Result<(), Self::Error> {
        let range = Range(address, Address(address.0 + bytes.len() as u32));
        if address.0 % 4 != 0 {
            Err(nb::Error::Other(Error::MisalignedAccess))
        } else if !range.is_writable() {
            return Err(nb::Error::Other(Error::MemoryNotReachable));
        } else {
            let base = address.0 as *const u8;
            for (index, byte) in bytes.iter_mut().enumerate() {
                // NOTE(Safety) we are reading directly from raw memory locations,
                // which is inherently unsafe. In this case, safety is guaranteed
                // because we can only read main memory blocks that don't contain
                // the bootloader image, and any direct write to them is handled through
                // a mutable reference to this same Flash struct, so there can't be
                // a data race.
                *byte = unsafe { *(base.add(index)) };
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ranges_span_the_correct_sectors() {
        let range = Range(Address(0x0801_1234), Address(0x0804_5678));
        let expected_sectors = &MEMORY_MAP.sectors[4..7];

        assert_eq!(expected_sectors, range.span());
    }
}
