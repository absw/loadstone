//! Utilities to manipulate generic memory

/// Generic address for the purpose of the
/// methods of this file. Anything that can be offset
/// by a usize and yield another  address works as an address.
pub trait Address: Copy + core::ops::Add<usize, Output = Self> {}
impl<A> Address for A where A: Copy + core::ops::Add<usize, Output = A> {}

/// Abstract sector that can contain addresses
pub trait Sector<A: Address> {
    fn contains(&self, address: A) -> bool;
}

/// Iterator producing block-sector pairs,
/// where each memory block corresponds to each sector
pub struct BlockAndSectorIterator<'a, A, S>
where
    A: Address,
    S: Sector<A>,
{
    memory: &'a [u8],
    sectors: &'a [S],
    base_address: A,
    sector_index: usize,
}

impl<'a, A, S> Iterator for BlockAndSectorIterator<'a, A, S>
where
    A: Address,
    S: Sector<A>,
{
    type Item = (&'a [u8], &'a S);

    fn next(&mut self) -> Option<Self::Item> {
        if self.sector_index >= self.sectors.len() {
            return None;
        }
        let current_sector = &self.sectors[self.sector_index];
        let mut block_range = (0..self.memory.len())
            .skip_while(|index| !current_sector.contains(self.base_address + *index))
            .take_while(|index| current_sector.contains(self.base_address + *index));
        let result = match (block_range.next(), block_range.last()) {
            (Some(start), Some(end)) if start < end => {
                Some((&self.memory[start..(end + 1)], current_sector))
            }
            _ => None,
        };
        self.sector_index += 1;
        result
    }
}

/// Anything that can be sliced in blocks, each block
/// corresponding to a sector in a sector sequence
pub trait IterableByBlocksAndSectors<'a, A, S>
where
    A: Address,
    S: Sector<A>,
{
    fn blocks_per_sector(
        &'a self,
        base_address: A,
        sectors: &'a [S],
    ) -> BlockAndSectorIterator<A, S>;
}

/// Blanket implementation of block and sector iteration for slices of bytes
impl<'a, A, S> IterableByBlocksAndSectors<'a, A, S> for &'a [u8]
where
    A: Address,
    S: Sector<A>,
{
    fn blocks_per_sector(&self, base_address: A, sectors: &'a [S]) -> BlockAndSectorIterator<A, S> {
        BlockAndSectorIterator { memory: self, sectors, base_address, sector_index: 0 }
    }
}

#[cfg(not(target_arch = "arm"))]
#[doc(hidden)]
pub mod doubles {
    use super::*;
    pub type FakeAddress = usize;

    #[derive(Debug, PartialEq)]
    pub struct FakeSector {
        pub start: FakeAddress,
        pub size: usize,
    }

    impl Sector<FakeAddress> for FakeSector {
        fn contains(&self, address: FakeAddress) -> bool {
            (self.start <= address) && ((self.start + self.size) > address)
        }
    }
}

#[cfg(test)]
mod test {
    use super::{doubles::*, *};

    #[test]
    fn iterating_over_sectors() {
        // Given
        const MEMORY_SIZE: usize = 0x50;
        let memory = [0xFFu8; MEMORY_SIZE];
        let memory_slice = &memory[..];
        let base_address = 0x20;

        let sectors =
            [FakeSector { start: 0x30, size: 0x10 }, FakeSector { start: 0x40, size: 0x05 }];

        // When
        let pairs: Vec<_> = memory_slice.blocks_per_sector(base_address, &sectors).collect();

        // Then
        let (block, sector) = pairs[0];
        assert_eq!(block, &memory[0x10..0x20]);
        assert_eq!(sector, &sectors[0]);
        let (block, sector) = pairs[1];
        assert_eq!(block, &memory[0x20..0x25]);
        assert_eq!(sector, &sectors[1]);
    }
}
