//! Utilities to manipulate generic memory
#![macro_use]

#[macro_export]
macro_rules! kb {
    ($val:expr) => {
        $val * 1024
    };
}
#[macro_export]
macro_rules! mb {
    ($val:expr) => {
        $val * 1024 * 1024
    };
}

/// Generic address for the purpose of this module's methods.
/// Anything that can be offset by a usize and yield another
/// address works as an address.
pub trait Address: Copy + core::ops::Add<usize, Output = Self> {}
impl<A> Address for A where A: Copy + core::ops::Add<usize, Output = A> {}

/// Abstract region that can contain addresses
pub trait Region<A: Address> {
    fn contains(&self, address: A) -> bool;
}

/// Iterator producing block-region pairs,
/// where each memory block corresponds to each region
pub struct OverlapIterator<'a, A, R, I>
where
    A: Address,
    R: Region<A>,
    I: Iterator<Item = R>,
{
    memory: &'a [u8],
    regions: I,
    base_address: A,
}

/// Anything that can be sliced in blocks, each block
/// corresponding to a region in a region sequence
pub trait IterableByOverlaps<'a, A, R, I>
where
    A: Address,
    R: Region<A>,
    I: Iterator<Item = R>,
{
    fn overlaps(self, block: &'a [u8], base_address: A) -> OverlapIterator<A, R, I>;
}

impl<'a, A, R, I> Iterator for OverlapIterator<'a, A, R, I>
where
    A: Address,
    R: Region<A>,
    I: Iterator<Item = R>,
{
    type Item = (&'a [u8], R, A);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(region) = self.regions.next() {
            let mut block_range = (0..self.memory.len())
                .skip_while(|index| !region.contains(self.base_address + *index))
                .take_while(|index| region.contains(self.base_address + *index));
            if let Some(start) = block_range.next() {
                let end = block_range.last().unwrap_or(start) + 1;
                return Some((&self.memory[start..end], region, self.base_address + start));
            }
        }
        None
    }
}

/// Blanket implementation of overlaps iterator for any region iterator
impl<'a, A, R, I> IterableByOverlaps<'a, A, R, I> for I
where
    A: Address,
    R: Region<A>,
    I: Iterator<Item = R>,
{
    fn overlaps(self, memory: &'a [u8], base_address: A) -> OverlapIterator<A, R, I> {
        OverlapIterator { memory, regions: self, base_address }
    }
}

#[cfg(not(target_arch = "arm"))]
#[doc(hidden)]
pub mod doubles {
    use super::*;
    pub type FakeAddress = usize;

    #[derive(Debug, PartialEq, Copy, Clone)]
    pub struct FakeRegion {
        pub start: FakeAddress,
        pub size: usize,
    }

    impl Region<FakeAddress> for FakeRegion {
        fn contains(&self, address: FakeAddress) -> bool {
            (self.start <= address) && ((self.start + self.size) > address)
        }
    }
}

#[cfg(test)]
mod test {
    use super::{doubles::*, *};

    #[test]
    fn iterating_over_regions_starting_before_them() {
        // Given
        const MEMORY_SIZE: usize = 0x50;
        let memory = [0xFFu8; MEMORY_SIZE];
        let memory_slice = &memory[..];
        let base_address = 0x20;

        let regions =
            [FakeRegion { start: 0x30, size: 0x10 }, FakeRegion { start: 0x40, size: 0x05 }];

        // When
        let pairs: Vec<_> = regions.iter().copied().overlaps(memory_slice, base_address).collect();

        // Then
        assert_eq!(pairs.len(), 2);

        let (block, region, address) = pairs[0];
        assert_eq!(block, &memory[0x10..0x20]);
        assert_eq!(region, regions[0]);
        assert_eq!(address, regions[0].start);
        let (block, region, address) = pairs[1];
        assert_eq!(block, &memory[0x20..0x25]);
        assert_eq!(region, regions[1]);
        assert_eq!(address, regions[1].start);
    }

    #[test]
    fn iterating_over_regions_starting_in_the_middle() {
        // Given
        const MEMORY_SIZE: usize = 30;
        let memory = [0; MEMORY_SIZE];
        let memory_slice = &memory[..];
        let base_address = 15;

        let regions = [FakeRegion { start: 10, size: 20 }, FakeRegion { start: 30, size: 100 }];

        // When
        let pairs: Vec<_> = regions.iter().copied().overlaps(memory_slice, base_address).collect();

        // Then
        assert_eq!(pairs.len(), 2);

        let (block, region, address) = pairs[0];
        assert_eq!(block, &memory[0..15]);
        assert_eq!(region, regions[0]);
        assert_eq!(address, base_address);

        let (block, region, address) = pairs[1];
        assert_eq!(block, &memory[15..30]);
        assert_eq!(region, regions[1]);
        assert_eq!(address, regions[1].start);
    }

    #[test]
    fn single_byte() {
        // Given
        const MEMORY_SIZE: usize = 1;
        let memory = [0; MEMORY_SIZE];
        let memory_slice = &memory[..];
        let base_address = 15;

        let regions = [FakeRegion { start: 10, size: 20 }, FakeRegion { start: 30, size: 100 }];

        // When
        let pairs: Vec<_> = regions.iter().copied().overlaps(memory_slice, base_address).collect();

        // Then
        assert_eq!(pairs.len(), 1);

        let (block, region, address) = pairs[0];
        assert_eq!(block, &memory[0..1]);
        assert_eq!(region, regions[0]);
        assert_eq!(address, base_address);
    }

    #[test]
    fn conversion_macros() {
        assert_eq!(kb!(16), 0x4000);
        assert_eq!(mb!(1), 0x100000);
    }
}
