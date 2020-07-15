//! Convenience bitwise operations.

use core::ops::BitOr;

/// Simple check for particular bits being set or cleared.
pub trait BitFlags {
    fn is_set(&self, bit: u8) -> bool;
    fn is_clear(&self, bit: u8) -> bool;
}

/// Checks that every '1' bit is a '1' on the
/// right hand side.
pub trait BitSubset: Copy {
    fn is_subset_of(self, rhs: Self) -> bool;
}

/// Variant of the BitSubset trait for slices.
pub trait SliceBitSubset {
    /// Checks that every '1' in self is '1' in T
    fn is_subset_of(self, rhs: Self) -> bool;
}

/// Blanket implementation for any types convertible to u32.
impl<U: Copy + Into<u32>> BitFlags for U {
    fn is_set(&self, bit: u8) -> bool {
        assert!(bit < 32);
        ((*self).into() & (1u32 << bit)) != 0
    }

    fn is_clear(&self, bit: u8) -> bool { !self.is_set(bit) }
}

impl<U: Copy + BitOr<Output = Self> + PartialEq> BitSubset for U {
    fn is_subset_of(self, rhs: Self) -> bool { (self | rhs) == rhs }
}

impl<T: BitSubset> SliceBitSubset for &[T] {
    fn is_subset_of(self, rhs: Self) -> bool {
        if self.len() > rhs.len() {
            false
        } else {
            self.iter().zip(rhs.iter()).all(|(a, b)| a.is_subset_of(*b))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn correctly_checks_bits() {
        assert!(3u8.is_set(0));
        assert!(3u8.is_set(1));
        assert!(3u8.is_clear(2));
        assert!(2u8.is_clear(0));
    }

    #[test]
    fn checks_bit_subsets() {
        assert!(0xAAu8.is_subset_of(0xFFu8));
        assert!(!0xFFFF_FFFF_u32.is_subset_of(0xAAAA_AAAA_u32));
        assert!(0b0101.is_subset_of(0b0111));
    }

    #[test]
    fn verify_memory_range_is_subset_of_sector() {
        let range = [0x12, 0x34, 0x56, 0x78];
        let newly_erased_sector = [0xFF, 0xFF, 0xFF, 0xFF];
        assert!(range.is_subset_of(&newly_erased_sector));
        assert!(!newly_erased_sector.is_subset_of(&range));

        let third_bits = [0b0100, 0b0100, 0b0100, 0b0100, 0b0100];
        let even_bits = [0b0101, 0b0101, 0b0101, 0b0101, 0b0101];
        assert!(third_bits.is_subset_of(&even_bits));
        assert!(!even_bits.is_subset_of(&third_bits));

        let short_range = [0xFF];
        let long_sector = [0xFF, 0xFF];
        assert!(short_range.is_subset_of(&long_sector));
        assert!(!long_sector.is_subset_of(&short_range));
    }
}
