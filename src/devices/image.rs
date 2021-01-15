use crate::error::Error;
use blue_hal::{
    hal::flash::{self, UnportableDeserialize, UnportableSerialize},
    utilities::memory::Address,
};
use core::{cmp::min, mem::size_of};
use crc::{crc32, Hasher32};
use nb::{self, block};

pub(crate) const TRANSFER_BUFFER_SIZE: usize = 2048usize;

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
    pub size: usize,
    pub crc: u32,
    pub name: Option<[u8; 32]>,
}

/// Image bank descriptor
#[derive(Clone)]
pub struct Bank<A: Address> {
    pub index: u8,
    pub size: usize,
    pub location: A,
    pub bootable: bool,
}

impl GlobalHeader {
    /// Attempts to retrieve a header from flash.
    pub fn retrieve<F, A>(flash: &mut F) -> Result<Self, Error>
    where
        A: Address,
        F: flash::ReadWrite<Address = A>,
        Error: From<F::Error>,
    {
        // Global header is always at the end of the readable region
        let address = flash.range().1 - size_of::<Self>();

        // NOTE(Safety): It is safe to deserialize here since we're checking the magic number for
        // validity. It will only cause UB when the structs in this file have been modified AND the
        // magic value at the top has not.
        let header: Self = block!(unsafe { flash.deserialize(address) })?;
        if header.magic == MAGIC {
            Ok(header)
        } else {
            Err(Error::FlashCorrupted)
        }
    }

    /// Writes a default global header to flash at the right location.
    pub fn format_default<F, A>(flash: &mut F) -> Result<(), Error>
    where
        A: Address,
        F: flash::ReadWrite<Address = A>,
        Error: From<F::Error>,
    {
        let default_header = Self { magic: MAGIC, test_buffer: [0x00; 4] };
        // Global header is always at the end of the readable region
        let address = flash.range().1 - size_of::<Self>();

        // NOTE(Safety): It is safe to serialize here since the type is defined in this file, and
        // we guarantee it doesn't contain references, that it's repr C, and that it will be stored
        // alongside a magic number that guarantees its safe retrieval from flash.
        Ok(block!(unsafe { flash.serialize(&default_header, address) })?)
    }
}

impl ImageHeader {
    /// Retrieves an image header from a given bank in flash memory.
    pub fn retrieve<F, A>(flash: &mut F, bank: &Bank<A>) -> Result<Self, Error>
    where
        A: Address,
        F: flash::ReadWrite<Address = A>,
        Error: From<F::Error>,
    {
        // Image headers are stored at the *end* of images to make sure the binary is aligned
        let address = bank.location + bank.size;
        // NOTE(Safety): It is safe to deserialize here since we're checking the magic number for
        // validity. It will only cause UB when the structs in this file have been modified AND the
        // magic value at the top has not.
        let header: Self = block!(unsafe { flash.deserialize(address) })?;
        if header.magic == MAGIC {
            Ok(header)
        } else {
            Err(Error::FlashCorrupted)
        }
    }

    /// Writes a default image header to flash at a given location
    pub fn format_default<A, F>(flash: &mut F, bank: &Bank<A>) -> Result<(), Error>
    where
        A: Address,
        F: flash::ReadWrite<Address = A>,
        Error: From<F::Error>,
    {
        // Image headers are stored at the *end* of images to make sure the binary is aligned
        let address = bank.location + bank.size;
        let default_header = Self { magic: MAGIC, size: 0, crc: 0, name: None };
        // NOTE(Safety): It is safe to serialize here since the type is defined in this file, and
        // we guarantee it doesn't contain references, that it's repr C, and that it will be stored
        // alongside a magic number that guarantees its safe retrieval from flash.
        Ok(block!(unsafe { flash.serialize(&default_header, address) })?)
    }

    /// Attempts to write a header to a given bank of flash.
    pub fn write<A, F>(flash: &mut F, bank: &Bank<A>, size: usize) -> Result<(), Error>
    where
        A: Address,
        F: flash::ReadWrite<Address = A>,
        Error: From<F::Error>,
    {
        // Header can **only ever** be written if the image is valid.
        let crc = Self::validate_image(flash, bank.location, size)?;
        // Image headers are stored at the *end* of images to make sure the binary is aligned
        let address = bank.location + bank.size;
        let header = ImageHeader {
            name: None, // TODO support named images
            magic: MAGIC,
            size,
            crc,
        };

        block!(unsafe { flash.serialize(&header, address) })?;
        if bank.sanity_check(flash).is_err() {
            Self::format_default(flash, bank).expect("FATAL: Flash unrecoverably corrupted");
            return Err(Error::FlashCorrupted);
        }
        Ok(())
    }

    /// Performs a CRC check on a given image.
    pub fn validate_image<A, F>(flash: &mut F, location: A, size: usize) -> Result<u32, Error>
    where
        A: Address,
        F: flash::ReadWrite<Address = A>,
        Error: From<F::Error>,
    {
        if size <= size_of::<u32>() {
            return Err(Error::BankEmpty);
        }

        let mut buffer = [0u8; TRANSFER_BUFFER_SIZE];
        let mut byte_index = 0usize;
        let mut digest = crc32::Digest::new(crc32::IEEE);
        let size_before_crc = size.saturating_sub(size_of::<u32>());

        while byte_index < size_before_crc {
            let remaining_size = size_before_crc.saturating_sub(byte_index);
            let bytes_to_read = min(TRANSFER_BUFFER_SIZE, remaining_size);
            let slice = &mut buffer[0..bytes_to_read];
            block!(flash.read(location + byte_index, slice))?;
            digest.write(slice);
            byte_index += bytes_to_read;
        }

        let mut crc_bytes = [0u8; 4];
        block!(flash.read(location + byte_index, &mut crc_bytes))?;
        let crc = u32::from_le_bytes(crc_bytes);
        let calculated_crc = digest.sum32();
        if crc == calculated_crc {
            Ok(crc)
        } else {
            Err(Error::CrcInvalid)
        }
    }
}

impl<A: Address> Bank<A> {
    /// Ensures that a bank's CRC is still valid and reflects the image within.
    pub fn sanity_check<F>(&self, flash: &mut F) -> Result<(), Error>
    where
        F: flash::ReadWrite<Address = A>,
        Error: From<F::Error>,
    {
        let header = ImageHeader::retrieve(flash, self)?;
        let header_crc = header.crc;
        let crc_location = self.location + header.size - size_of::<u32>();
        let mut crc_bytes = [0u8; 4];
        block!(flash.read(crc_location, &mut crc_bytes))?;
        let stored_crc = u32::from_le_bytes(crc_bytes);
        if header_crc == stored_crc {
            Ok(())
        } else {
            Err(Error::CrcInvalid)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use blue_hal::hal::{
        doubles::{
            error::FakeError,
            flash::{Address, FakeFlash},
        },
        flash::ReadWrite,
    };

    impl From<FakeError> for Error {
        fn from(_: FakeError) -> Self { Error::DeviceError("Something fake happened") }
    }

    #[test]
    fn writing_header_with_correct_crc() {
        // Given
        let mut flash = FakeFlash::new(Address(0));
        let bank = Bank { index: 1, size: 512, location: Address(0), bootable: false };
        let image_with_crc = [0xAAu8, 0xBB, /*CRC*/ 0x98, 0x2c, 0x82, 0x49];
        flash.write(Address(0), &image_with_crc).unwrap();

        // Then
        ImageHeader::write(&mut flash, &bank, image_with_crc.len()).unwrap();
    }

    #[test]
    fn attempting_to_write_header_with_wrong_crc() {
        // Given
        let mut flash = FakeFlash::new(Address(0));
        let bank = Bank { index: 1, size: 512, location: Address(0), bootable: false };
        let image_with_crc = [0xAAu8, 0xBB, /*CRC*/ 0x01, 0x02, 0x03, 0x04];
        flash.write(Address(0), &image_with_crc).unwrap();

        // Then
        let _result = ImageHeader::write(&mut flash, &bank, image_with_crc.len()).unwrap_err();
        assert!(matches!(Error::CrcInvalid, _result));
    }
}
