use crate::error::Error;
use core::mem::size_of;

use super::*;
use blue_hal::{
    hal::flash,
    utilities::{iterator::UntilSequence, memory::Address},
};
use crc::{crc32, Hasher32};
use nb::block;

/// Scans a bank to determine the presence of a valid, crc-verified firmware image. If
/// successful, returns the [descriptor](`Image<A>`) for that image.
pub fn image_at<A, F>(flash: &mut F, bank: Bank<A>) -> Result<Image<A>, Error>
where
    A: Address,
    F: flash::ReadWrite<Address = A>,
    Error: From<F::Error>,
{
    // Generic buffer to hold temporary slices read from flash memory.
    const BUFFER_SIZE: usize = 256;
    let mut buffer = [0u8; BUFFER_SIZE];

    let (mut digest, mut image_size) = flash
        .bytes(bank.location)
        .take(bank.size)
        .until_sequence(&magic_string_inverted())
        .fold((crc32::Digest::new(crc32::IEEE), 0usize), |(mut digest, mut byte_count), byte| {
            digest.write(&[byte]);
            byte_count += 1;
            (digest, byte_count)
        });

    if image_size == bank.size {
        return Err(Error::BankEmpty);
    }

    // Magic string is part of the digest
    digest.write(&magic_string_inverted());
    let digest_position = bank.location + image_size + MAGIC_STRING.len();
    let mut digest_bytes = [0; size_of::<u32>()];
    block!(flash.read(digest_position, &mut digest_bytes))?;

    let retrieved_crc = u32::from_le_bytes(digest_bytes);
    let calculated_crc = digest.sum32();
    if retrieved_crc != calculated_crc {
        return Err(Error::CrcInvalid);
    }

    let golden_string_position = bank.location + image_size.saturating_sub(GOLDEN_STRING.len());
    let golden_bytes = &mut buffer[0..GOLDEN_STRING.len()];
    block!(flash.read(golden_string_position, golden_bytes))?;
    let golden = golden_bytes == GOLDEN_STRING.as_bytes();

    if golden {
        image_size = image_size.saturating_sub(GOLDEN_STRING.len());
    }

    Ok(Image {
        size: image_size,
        location: bank.location,
        bootable: bank.bootable,
        golden,
        crc: calculated_crc,
    })
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use super::*;
    use blue_hal::hal::{
        doubles::{
            error::FakeError,
            flash::{Address, FakeFlash},
        },
        flash::ReadWrite,
    };

    #[rustfmt::skip]
    const TEST_IMAGE_WITH_CORRECT_CRC: &[u8] = &[
        // Image
        0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x77, 0x6f, 0x72, 0x6c, 0x64, 0x0a,
        // Magic string inverted
        0xb7, 0xac, 0x9c, 0xc8, 0x9c, 0xcd, 0x8f, 0x8b,
        0x86, 0x9b, 0xa5, 0xb7, 0xcd, 0xae, 0x94, 0x8e, 0xa5, 0xa8,
        0xaf, 0x9c, 0xb5, 0x98, 0xb8, 0xcc, 0xb5, 0x8b, 0x91, 0xb5,
        0xc9, 0xa9, 0x8a, 0xbe,
        // CRC
        0xf0, 0xc9, 0x42, 0xad
    ];

    #[rustfmt::skip]
    const TEST_IMAGE_WITH_BAD_CRC: &[u8] = &[
        // Image
        0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x77, 0x6f, 0x72, 0x6c, 0x64, 0x0a,
        // Magic string inverted
        0xb7, 0xac, 0x9c, 0xc8, 0x9c, 0xcd, 0x8f, 0x8b,
        0x86, 0x9b, 0xa5, 0xb7, 0xcd, 0xae, 0x94, 0x8e, 0xa5, 0xa8,
        0xaf, 0x9c, 0xb5, 0x98, 0xb8, 0xcc, 0xb5, 0x8b, 0x91, 0xb5,
        0xc9, 0xa9, 0x8a, 0xbe,
        // CRC (first byte invalid)
        0x77, 0xc9, 0x42, 0xad
    ];

    #[test]
    fn retrieving_image_with_correct_crc_succeeds() {
        let mut flash = FakeFlash::new(Address(0));
        let bank =
            Bank { index: 1, size: 512, location: Address(0), bootable: false, is_golden: false };
        flash.write(Address(0), &TEST_IMAGE_WITH_CORRECT_CRC).unwrap();

        let image = image_at(&mut flash, bank).unwrap();
        assert_eq!(image.size, 12usize);
        assert_eq!(image.location, bank.location);
        assert_eq!(image.bootable, false);
        assert_eq!(image.is_golden(), false);
    }

    #[test]
    fn retrieving_image_with_incorrect_crc_fails() {
        let mut flash = FakeFlash::new(Address(0));
        let bank =
            Bank { index: 1, size: 512, location: Address(0), bootable: false, is_golden: false };

        flash.write(Address(0), &TEST_IMAGE_WITH_BAD_CRC).unwrap();
        assert_eq!(Err(Error::CrcInvalid), image_at(&mut flash, bank));
    }
}
