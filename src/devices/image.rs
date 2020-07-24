use crate::{
    error::Error,
    hal::flash::{self, UnportableDeserialize, UnportableSerialize},
    utilities::memory::Address,
};
use core::mem::size_of;
use nb::{self, block};

/// Changing this magic will force any headers in
/// flash memory to be considered incorrect. Change it
/// whenever you make a modification to any of the
/// header types below.
pub const MAGIC: u32 = 0xAB3ACADA;

// If MAGIC was changed to be 0xFFFF_FFFF, uninitialized
// flash memory would be confused with valid memory.
static_assertions::const_assert!(MAGIC != 0xFFFF_FFFF);

/// Header present at the lowest writable
/// address of a flash chip (either MCU or external).
#[repr(C)]
pub struct GlobalHeader {
    magic: u32,
    /// Scratchpad buffer to test arbitrary read/write operations.
    /// The value persisted here is meaningless.
    pub test_buffer: [u8; 4],
}

/// Header present at the start of any firmware image.
#[repr(C)]
pub struct ImageHeader {
    magic: u32,
    size: usize,
    crc: u32,
    name: Option<[u8; 32]>,
}

/// Image bank descriptor
pub struct Bank<A: Address> {
    pub size: usize,
    pub location: A,
}

impl GlobalHeader {
    pub fn retrieve<F: flash::ReadWrite>(flash: &mut F) -> nb::Result<Self, Error> {
        // Global header is always at the bottom of the readable region
        let (address, _) = F::range();

        // NOTE(Safety): It is safe to deserialize here since we're checking the magic number for
        // validity. It will only cause UB when the structs in this file have been modified AND the
        // magic value at the top has not.
        let header: Self = block!(unsafe { flash.deserialize(address) })
            .map_err(|_| Error::DriverError("Flash Read Failed"))?;
        if header.magic == MAGIC {
            Ok(header)
        } else {
            Err(nb::Error::Other(Error::LogicError(
                "Attempted to read global header from a corrupted/outdated flash.",
            )))
        }
    }

    // Writes a default global header to flash at the right location.
    pub fn format_default<F: flash::ReadWrite>(flash: &mut F) -> nb::Result<(), Error> {
        let default_header = Self { magic: MAGIC, test_buffer: [0x00; 4] };
        let (address, _) = F::range();

        // NOTE(Safety): It is safe to serialize here since the type is defined in this file, and
        // we guarantee it doesn't contain references, that it's repr C, and that it will be stored
        // alongside a magic number that guarantees its safe retrieval from flash.
        block!(unsafe { flash.serialize(&default_header, address) })
            .map_err(|_| nb::Error::Other(Error::DriverError("Flash Write Failed")))
    }
}

impl ImageHeader {
    pub fn retrieve<A: Address, F: flash::ReadWrite<Address = A>>(
        flash: &mut F,
        address: A,
    ) -> nb::Result<Self, Error> {
        let (global_header_address, _) = F::range();
        // No matter what, a header can't overlap with the global header.
        if address < (global_header_address + size_of::<GlobalHeader>()) {
            return Err(nb::Error::Other(Error::LogicError(
                "Attempted to read misplaced image header",
            )));
        }

        // NOTE(Safety): It is safe to deserialize here since we're checking the magic number for
        // validity. It will only cause UB when the structs in this file have been modified AND the
        // magic value at the top has not.
        let header: Self = block!(unsafe { flash.deserialize(address) })
            .map_err(|_| Error::DriverError("Flash Read Failed"))?;
        if header.magic == MAGIC {
            Ok(header)
        } else {
            Err(nb::Error::Other(Error::LogicError(
                "Attempted to read global header from a corrupted/outdated flash.",
            )))
        }
    }

    // Writes a default image header to flash at a given location
    pub fn format_default<A: Address, F: flash::ReadWrite<Address = A>>(flash: &mut F, address: A) -> nb::Result<(), Error> {
        let (global_header_address, _) = F::range();

        // No matter what, a header can't overlap with the global header.
        if address < (global_header_address + size_of::<GlobalHeader>()) {
            return Err(nb::Error::Other(Error::LogicError(
                "Attempted to write misplaced image header",
            )));
        }
        let default_header = Self { magic: MAGIC, size: 0, crc: 0, name: None};
        // NOTE(Safety): It is safe to serialize here since the type is defined in this file, and
        // we guarantee it doesn't contain references, that it's repr C, and that it will be stored
        // alongside a magic number that guarantees its safe retrieval from flash.
        block!(unsafe { flash.serialize(&default_header, address) })
            .map_err(|_| nb::Error::Other(Error::DriverError("Flash Write Failed")))
    }
}
