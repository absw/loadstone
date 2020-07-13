/// Abstract mass erase
pub trait BulkErase {
    type Error;
    fn erase(&mut self) -> nb::Result<(), Self::Error>;
}

/// Reads a range of bytes, generic over an address
pub trait Read<A> {
    type Error;
    fn read(&mut self, address: A, bytes: &mut [u8]) -> nb::Result<(), Self::Error>;
    fn readable_range() -> (A, A);
}

/// Writes a range of bytes, generic over an address
/// This is a high level write that abstracts away
/// the need to first erase, or to keep writes inside
/// page boundaries
pub trait Write<A> {
    type Error;
    fn write(&mut self, address: A, bytes: &[u8]) -> nb::Result<(), Self::Error>;
    fn writable_range() -> (A, A);
}
