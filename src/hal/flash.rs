use core::fmt;

/// Abstract mass erase
pub trait BulkErase {
    type Error;
    fn erase(&mut self) -> nb::Result<(), Self::Error>;
}

/// Reads a range of bytes, generic over an address
pub trait Read {
    type Error: Clone + Copy + fmt::Debug;
    type Address: Clone + Copy + fmt::Debug;
    fn read(&mut self, address: Self::Address, bytes: &mut [u8]) -> nb::Result<(), Self::Error>;
    fn readable_range() -> (Self::Address, Self::Address);
}

/// Writes a range of bytes, generic over an address
/// This is a high level write that abstracts away
/// the need to first erase, or to keep writes inside
/// page boundaries
pub trait Write {
    type Error: Clone + Copy + fmt::Debug;
    type Address: Clone + Copy + fmt::Debug;
    fn write(&mut self, address: Self::Address, bytes: &[u8]) -> nb::Result<(), Self::Error>;
    fn writable_range() -> (Self::Address, Self::Address);
}

pub trait ReadWrite: Read + Write {}
impl<F> ReadWrite for F where F: Read + Write {}
