use crate::utilities::memory::Address;
use core::{
    fmt,
    mem::{size_of, MaybeUninit},
    slice,
};

/// Abstract mass erase
pub trait BulkErase {
    type Error;
    fn erase(&mut self) -> nb::Result<(), Self::Error>;
}

/// Reads a range of bytes, generic over an address
pub trait Read {
    type Error: Clone + Copy + fmt::Debug;
    type Address: Address;
    fn read(&mut self, address: Self::Address, bytes: &mut [u8]) -> nb::Result<(), Self::Error>;
    fn readable_range() -> (Self::Address, Self::Address);
}

/// Writes a range of bytes, generic over an address
/// This is a high level write that abstracts away
/// the need to first erase, or to keep writes inside
/// page boundaries
pub trait Write {
    type Error: Clone + Copy + fmt::Debug;
    type Address: Address;
    fn write(&mut self, address: Self::Address, bytes: &[u8]) -> nb::Result<(), Self::Error>;
    fn writable_range() -> (Self::Address, Self::Address);
}

pub trait ReadWrite: Read + Write {
    type Address: Address;
}

impl<F> ReadWrite for F
where
    F: Read<Address = <Self as Write>::Address> + Write,
{
    type Address = <Self as Write>::Address;
}

pub trait UnportableSerialize: Write {
    /// NOTE(Safety): This is a very raw serialization (the bytes are written as-is). Should be
    /// only used with repr(C) types with no internal references. It *will break* if any change
    /// to the struct to serialize is made between serialization and deserialization, and it
    /// *will* cause undefined behaviour. Make sure to erase the flash whenever there is an
    /// update to the serializable types.
    unsafe fn serialize<T: Sized>(
        &mut self,
        item: &T,
        address: Self::Address,
    ) -> nb::Result<(), <Self as Write>::Error> {
        // Get a view into the raw bytes conforming T
        let bytes = slice::from_raw_parts((item as *const T) as *const u8, size_of::<T>());
        self.write(address, bytes)
    }
}
impl<F: Write> UnportableSerialize for F {}

pub trait UnportableDeserialize: Read {
    /// NOTE(Safety): This is a very raw serialization (the bytes are written as-is). Should be
    /// only used in repr(C) types. It *will break* if any change to the struct to serialize is
    /// made between serialization and deserialization, and it *will* cause undefined
    /// behaviour. Make sure to erase the flash whenever there is an update to the serializable
    /// types.
    unsafe fn deserialize<T: Sized>(
        &mut self,
        address: Self::Address,
    ) -> nb::Result<T, <Self as Read>::Error> {
        // Create uninitialized T with a zero repr
        let mut uninit: MaybeUninit<T> = MaybeUninit::uninit();
        let bytes = slice::from_raw_parts_mut(uninit.as_mut_ptr() as *mut _, size_of::<T>());

        // Read its byte representation into it
        self.read(address, bytes)?;
        Ok(uninit.assume_init())
    }
}
impl<F: Read> UnportableDeserialize for F {}
