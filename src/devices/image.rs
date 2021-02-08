use blue_hal::{
    hal::flash,
    utilities::{buffer::CollectSlice, iterator::UntilSequence, memory::Address},
};
use crc::{crc32, Hasher32};
use nb::{self, block};

use crate::error::Error;

/// This string precedes the CRC for golden images only
pub const GOLDEN_STRING: &str = "XPIcbOUrpG";
/// This string, INVERTED BYTEWISE must terminate any valid images, after CRC
///
/// Note: Why inverted? Because if we used it as-is, no code that includes this
/// constant could be used as a firmware image, as it contains the magic string
/// halfway through.
pub const MAGIC_STRING: &str = "HSc7c2ptydZH2QkqZWPcJgG3JtnJ6VuA";
pub fn magic_string_inverted() -> [u8; MAGIC_STRING.len()] {
    let mut inverted = [0u8; MAGIC_STRING.len()];
    let mut bytes = MAGIC_STRING.as_bytes().iter().map(|b| !b);
    bytes.collect_slice(&mut inverted);
    inverted
}
pub const CRC_SIZE_BYTES: usize = 4;

#[derive(Clone, Copy, Debug)]
pub struct Bank<A: Address> {
    pub index: u8,
    pub size: usize,
    pub location: A,
    pub bootable: bool,
    pub is_golden: bool,
}

/// Image descriptor
#[derive(Clone, Debug, PartialEq)]
pub struct Image<A: Address> {
    size: usize,
    location: A,
    bootable: bool,
    golden: bool,
    crc: u32,
}

impl<A: Address> Image<A> {
    pub fn location(&self) -> A { self.location }
    pub fn size(&self) -> usize { self.size }
    pub fn is_golden(&self) -> bool { self.golden }
    pub fn crc(&self) -> u32 { self.crc }
}

pub fn image_at<A, F>(flash: &mut F, bank: Bank<A>) -> Result<Image<A>, Error>
where
    A: Address,
    F: flash::ReadWrite<Address = A>,
    Error: From<F::Error>,
{
    // Development build shorcut: We're checking that the image does *not* start with 0xFF. This
    // will not be part of the final Loadstone release build, but it helps speed up the
    // verification for invalid images during development.
    if flash.bytes(bank.location).next().ok_or(Error::BankInvalid)? == 0xFF {
        return Err(Error::BankEmpty);
    }

    // TODO optimise this away so we don't have to scan the image twice. (e.g. with a "window"
    // buffer for the CRC);
    let image_size_with_crc =
        flash.bytes(bank.location).take(bank.size).until_sequence(&magic_string_inverted()).count();
    let image_size = image_size_with_crc.saturating_sub(CRC_SIZE_BYTES);
    let image_bytes = flash.bytes(bank.location).take(image_size);
    let digest = image_bytes.fold(crc32::Digest::new(crc32::IEEE), |mut digest, byte| {
        digest.write(&[byte]);
        digest
    });
    let calculated_crc = digest.sum32();
    let crc_offset = image_size;
    let mut crc_bytes = [0u8; CRC_SIZE_BYTES];
    block!(flash.read(bank.location + crc_offset, &mut crc_bytes))?;
    let crc = u32::from_le_bytes(crc_bytes);

    let golden_string_offset = crc_offset.saturating_sub(GOLDEN_STRING.len());
    let mut golden_bytes = [0u8; GOLDEN_STRING.len()];
    block!(flash.read(bank.location + golden_string_offset, &mut golden_bytes))?;
    let golden = golden_bytes == GOLDEN_STRING.as_bytes();

    if crc == calculated_crc {
        Ok(Image {
            size: image_size,
            location: bank.location,
            bootable: bank.bootable,
            golden,
            crc,
        })
    } else {
        Err(Error::CrcInvalid)
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

    fn test_image_with_crc() -> [u8; 38] {
        let mut array = [0u8; 38];
        array[..2].copy_from_slice(&[0xAAu8, 0xBB]); // Image
        array[2..34].copy_from_slice(&magic_string_inverted());
        array[34..].copy_from_slice(&[0x98, 0x2c, 0x82, 0x49]); // CRC
        array
    }

    #[test]
    fn retrieving_broken_image_fails() {
        let mut flash = FakeFlash::new(Address(0));
        let bank =
            Bank { index: 1, size: 512, location: Address(0), bootable: false, is_golden: false };
        let mut image_with_crc = test_image_with_crc();
        image_with_crc[1] = 0xFF; // This will corrupt the image, making the CRC obsolete
        flash.write(Address(0), &image_with_crc).unwrap();
        assert_eq!(Err(Error::CrcInvalid), image_at(&mut flash, bank));

        let bank =
            Bank { index: 1, size: 512, location: Address(0), bootable: false, is_golden: false };
        let mut image_with_crc = test_image_with_crc();
        image_with_crc[4] = 0xFF; // This will break the CRC directly
        flash.write(Address(0), &image_with_crc).unwrap();
        assert_eq!(Err(Error::CrcInvalid), image_at(&mut flash, bank));

        let bank =
            Bank { index: 1, size: 512, location: Address(0), bootable: false, is_golden: false };
        let mut image_with_crc = test_image_with_crc();
        image_with_crc[12] = 0xFF; // The magic string is not present to delineate the image
        flash.write(Address(0), &image_with_crc).unwrap();
        assert_eq!(Err(Error::CrcInvalid), image_at(&mut flash, bank));
    }
}
