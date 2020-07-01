/// Simple check for particular bits being set or cleared
pub trait BitFlags {
    fn is_set(&self, bit: u8) -> bool;
    fn is_clear(&self, bit: u8) -> bool;
}

/// Blanket implementation for any types convertible to u32
impl<U: Copy + Into<u32>> BitFlags for U {
    fn is_set(&self, bit: u8) -> bool {
        assert!(bit < 32);
        (*self).into() | (1u32 << bit) != 0
    }

    fn is_clear(&self, bit: u8) -> bool {
        assert!(bit < 32);
        (*self).into() & !(1u32 << bit) != 0
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

}
