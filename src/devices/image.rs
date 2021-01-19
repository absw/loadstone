use blue_hal::{hal::flash, utilities::{iterator::UntilSequence, memory::Address}};
use crc::{crc32, Hasher32};
use defmt::info;
use nb::{self, block};

use crate::error::Error;

/// This string must terminate any valid images, followed by CRC
pub const MAGIC_STRING: &str = "HSc7c2ptydZH2QkqZWPcJgG3JtnJ6VuA";
pub const CRC_SIZE_BYTES: usize = 4;

#[derive(Clone, Copy, Debug)]
pub struct Bank<A: Address> {
    pub index: u8,
    pub size: usize,
    pub location: A,
    pub bootable: bool,
}

/// Image descriptor
#[derive(Clone, Debug, PartialEq)]
pub struct Image<A: Address> {
    size: usize,
    location: A,
    bootable: bool,
}

impl<A: Address> Image<A> {
    pub fn size(&self) -> usize { self.size }
}

pub fn image_at<A, F>(flash: &mut F, bank: Bank<A>) -> Result<Image<A>, Error>
where
    A: Address,
    F: flash::ReadWrite<Address = A>,
    Error: From<F::Error>,
{
    let bytes = flash.bytes(bank.location).take(bank.size).until_sequence(MAGIC_STRING.as_bytes());
    info!("Verifying image...");
    let (size, digest) = bytes.fold((0, crc32::Digest::new(crc32::IEEE)), |(mut size, mut digest), byte| {
        size += 1;
        digest.write(&[byte]);
        (size, digest)
    });
    info!("Done verifying image.");
    let calculated_crc = digest.sum32();

    let crc_offset = size + MAGIC_STRING.len();
    let mut crc_bytes = [0u8; CRC_SIZE_BYTES];
    block!(flash.read(bank.location + crc_offset, &mut crc_bytes))?;
    let crc = u32::from_le_bytes(crc_bytes);
    if crc == calculated_crc {
        Ok(Image {
            size,
            location: bank.location,
            bootable: bank.bootable
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
        array[2..34].copy_from_slice(MAGIC_STRING.as_bytes());
        array[34..].copy_from_slice(&[0x98, 0x2c, 0x82, 0x49,]); // CRC
        array
    }

    #[test]
    fn retrieving_image_from_flash() {
        let mut flash = FakeFlash::new(Address(0));
        let bank = Bank { index: 1, size: 512, location: Address(0), bootable: false };
        let image_with_crc = test_image_with_crc();
        flash.write(Address(0), &image_with_crc).unwrap();
        assert_eq!(Ok(Image {
            size: 2,
            location: bank.location,
            bootable: bank.bootable
        }), image_at(&mut flash, bank));
    }

    #[test]
    fn retrieving_broken_image_fails() {
        let mut flash = FakeFlash::new(Address(0));
        let bank = Bank { index: 1, size: 512, location: Address(0), bootable: false };
        let mut image_with_crc = test_image_with_crc();
        image_with_crc[0] = 0xFF; // This will corrupt the image, making the CRC obsolete
        flash.write(Address(0), &image_with_crc).unwrap();
        assert_eq!(Err(Error::CrcInvalid), image_at(&mut flash, bank));

        let bank = Bank { index: 1, size: 512, location: Address(0), bootable: false };
        let mut image_with_crc = test_image_with_crc();
        image_with_crc[4] = 0xFF; // This will break the CRC directly
        flash.write(Address(0), &image_with_crc).unwrap();
        assert_eq!(Err(Error::CrcInvalid), image_at(&mut flash, bank));

        let bank = Bank { index: 1, size: 512, location: Address(0), bootable: false };
        let mut image_with_crc = test_image_with_crc();
        image_with_crc[12] = 0xFF; // The magic string is not present to delineate the image
        flash.write(Address(0), &image_with_crc).unwrap();
        assert_eq!(Err(Error::CrcInvalid), image_at(&mut flash, bank));
    }
}
