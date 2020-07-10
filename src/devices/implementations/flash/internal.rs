//! Internal Flash controller for the STM32F4 family
use crate::{devices::interfaces::flash::Write, stm32pac::FLASH};
use static_assertions::const_assert;
use nb::block;

pub struct InternalFlash {
    flash: FLASH,
}

pub enum Error {
    MemoryNotWrittable,
    MisalignedAccess,
}

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq)]
pub struct Address(u32);

#[derive(Copy, Clone, Debug)]
struct Range(Address, Address);

#[derive(Copy, Clone, Debug, PartialEq)]
enum Block {
    Boot, // Main memory, but reserved for secure bootloader
    Main,
    SystemMemory,
    OtpArea,
    OptionBytes,
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct Sector {
    block: Block,
    start: Address,
    end: Address,
}

struct MemoryMap {
    sectors: &'static [Sector],
}

///From [section 3.5.1](../../../../../../../../documentation/hardware/stm32f412_reference.pdf#page=62)
const UNLOCK_KEYS: [u32; 2] = [0x45670123, 0xCDEF89AB];

// Compile time check that the memory map below is correct.
const_assert!(MEMORY_MAP.is_sound());

#[cfg(feature = "stm32f412")]
const MEMORY_MAP: MemoryMap = MemoryMap {
    sectors: &[
        Sector { block: Block::Boot, start: Address(0x0800_0000), end: Address(0x0800_4000) },
        Sector { block: Block::Boot, start: Address(0x0800_4000), end: Address(0x0800_8000) },
        Sector { block: Block::Boot, start: Address(0x0800_8000), end: Address(0x0800_C000) },
        Sector { block: Block::Boot, start: Address(0x0800_C000), end: Address(0x0801_0000) },
        Sector { block: Block::Main, start: Address(0x0801_0000), end: Address(0x0802_0000) },
        Sector { block: Block::Main, start: Address(0x0802_0000), end: Address(0x0804_0000) },
        Sector { block: Block::Main, start: Address(0x0804_0000), end: Address(0x0806_0000) },
        Sector { block: Block::Main, start: Address(0x0806_0000), end: Address(0x0808_0000) },
        Sector { block: Block::Main, start: Address(0x0808_0000), end: Address(0x080A_0000) },
        Sector { block: Block::Main, start: Address(0x080A_0000), end: Address(0x080C_0000) },
        Sector { block: Block::Main, start: Address(0x080C_0000), end: Address(0x080E_0000) },
        Sector { block: Block::Main, start: Address(0x080E_0000), end: Address(0x0810_0000) },
        Sector {
            block: Block::SystemMemory,
            start: Address(0x1FFF_0000),
            end: Address(0x1FFF_7800),
        },
        Sector { block: Block::OtpArea, start: Address(0x1FFF_7800), end: Address(0x1FFF_7A0F) },
        Sector {
            block: Block::OptionBytes,
            start: Address(0x1FFF_C000),
            end: Address(0x1FFF_C010),
        },
    ],
};

impl MemoryMap {
    // Verifies that the memory map is consecutive at compile time
    // NOTE: Some of the control flow here is necessarily awkward,
    // since this is a compile-time function and it doesn't have
    // access to complex constructs.
    const fn is_sound(&self) -> bool {
        // Verify all ranges are valid
        let mut index = 0usize;
        loop {
            if index == self.sectors.len() {
                break;
            }

            let range = Range(self.sectors[index].start, self.sectors[index].end);
            if !range.is_valid() {
                return false;
            }
            index = index + 1;
        }

        // Verify main memory is consecutive
        let mut index = 0usize;
        loop {
            let next = index + 1;
            if !self.sectors[next].is_in_main_memory_area() {
                break true;
            } else {
                let (end, start) = (self.sectors[index].end, self.sectors[next].start);
                if end.0 != start.0 {
                    break false;
                }
            }
            index = next;
        }
    }
}

impl Address {
    const fn is_inside(self, sector: &Sector) -> bool {
        self.0 >= sector.start.0 && self.0 < sector.end.0
    }

    fn sector(&self) -> Option<Sector> {
        MEMORY_MAP.sectors.iter().cloned().find(|sector| self.is_inside(&sector))
    }
}

impl Range {
    /// Sectors spanned by this range of addresses
    fn span(self) -> &'static [Sector] {
        let Range(start, end) = self;
        let mut span = MEMORY_MAP
            .sectors
            .iter()
            .enumerate()
            .skip_while(|(_, sector)| sector.end < start)
            .take_while(|(_, sector)| sector.start < end)
            .map(|(index, _)| index);
        let start = span.next().unwrap_or_default();
        let end = span.last().unwrap_or_default();
        &MEMORY_MAP.sectors[start..end]
    }

    const fn is_valid(self) -> bool {
        let Range(Address(start), Address(end)) = self;
        let after_map = start >= MEMORY_MAP.sectors[MEMORY_MAP.sectors.len() - 1].end.0;
        let before_map = end < MEMORY_MAP.sectors[0].start.0;
        let monotonic = end >= start;
        monotonic && !before_map && !after_map
    }

    /// Verify that all sectors spanned by this range are writable
    fn is_writable(self) -> bool { self.span().iter().all(Sector::is_writable) }
}

impl Sector {
    fn number(&self) -> Option<u8> {
        MEMORY_MAP.sectors.iter().enumerate().find_map(|(index, sector)| {
            (sector.is_in_main_memory_area() && self == sector).then_some(index as u8)
        })
    }

    const fn size(&self) -> usize { (self.end.0 - self.start.0) as usize }
    const fn is_writable(&self) -> bool { self.block as u8 == Block::Main as u8 }
    const fn is_in_main_memory_area(&self) -> bool {
        self.block as u8 == Block::Main as u8 || self.block as u8 == Block::Boot as u8
    }
}

impl InternalFlash {
    pub fn new(flash: FLASH) -> Result<Self, Error> { Ok(Self { flash }) }

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
        let number = sector.number().ok_or(nb::Error::Other(Error::MemoryNotWrittable))?;
        self.unlock()?;
        self.flash
            .cr
            .modify(|_, w| unsafe { w.ser().set_bit().snb().bits(number).strt().set_bit() });
        self.lock();
        Ok(())
    }

    fn is_busy(&self) -> bool { self.flash.sr.read().bsy().bit_is_set() }
}

impl Write<Address> for InternalFlash {
    type Error = Error;

    fn write(&mut self, address: Address, bytes: &[u8]) -> nb::Result<(), Self::Error> {
        if address.0 % 4 != 0 {
            return Err(nb::Error::Other(Error::MisalignedAccess))
        }

        // Adjust end for alignment
        let range = Range(address, Address(address.0 + bytes.len() as u32));
        if !range.is_writable() {
            return Err(nb::Error::Other(Error::MemoryNotWrittable));
        }

        // Early yield if busy
        if self.is_busy() {
            return Err(nb::Error::WouldBlock);
        }

        //TODO smart read-write cycle
        for sector in range.span() { block!(self.erase(sector))?; }

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
            unsafe { *base_address.offset(index as isize) = word; }
        }
        self.lock();
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ranges_span_the_correct_sectors() {
        let range = Range(Address(0x0801_1234), Address(0x0804_5678));
        let expected_sectors = &MEMORY_MAP.sectors[4..6];

        assert_eq!(expected_sectors, range.span());
    }
}
