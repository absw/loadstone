use ::ecdsa::{generic_array::typenum::Unsigned, SignatureSize};
use blue_hal::{
    hal::flash,
    utilities::{buffer::CollectSlice, iterator::UntilSequence, memory::Address},
};
use ecdsa::signature::Signature as EcdsaSignature;
use nb::{self, block};
use p256::{
    ecdsa::{signature::DigestVerifier, Signature, VerifyingKey},
    NistP256,
};
use sha2::Digest;

use crate::error::Error;
use core::str::FromStr;

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
    signature: Signature,
}

impl<A: Address> Image<A> {
    pub fn location(&self) -> A { self.location }
    pub fn size(&self) -> usize { self.size }
    pub fn is_golden(&self) -> bool { self.golden }
    pub fn signature(&self) -> Signature { self.signature }
}

pub fn retrieve_key() -> VerifyingKey {
    VerifyingKey::from_str(include_str!("assets/test_key.pem"))
        .expect("Invalic public key supplied on compilation")
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
    let key = retrieve_key();

    // Generic buffer to hold temporary slices read from flash memory.
    const BUFFER_SIZE: usize = 256;
    let mut buffer = [0u8; BUFFER_SIZE];

    let (mut digest, mut image_size) = flash
        .bytes(bank.location)
        .take(bank.size)
        .until_sequence(&magic_string_inverted())
        .fold((sha2::Sha256::default(), 0usize), |(mut digest, mut byte_count), byte| {
            digest.update(&[byte]);
            byte_count += 1;
            (digest, byte_count)
        });

    if image_size == bank.size {
        return Err(Error::BankEmpty);
    }

    // Magic string is part of the digest
    digest.update(&magic_string_inverted());

    let signature_position = bank.location + image_size + MAGIC_STRING.len();
    let signature_bytes = &mut buffer[0..SignatureSize::<NistP256>::to_usize()];
    block!(flash.read(signature_position, signature_bytes))?;

    let signature = Signature::from_bytes(signature_bytes).map_err(|_| Error::SignatureInvalid)?;
    key.verify_digest(digest, &signature).map_err(|_| Error::SignatureInvalid)?;

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
        signature,
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

    impl From<FakeError> for Error {
        fn from(_: FakeError) -> Self { Error::DeviceError("Something fake happened") }
    }

    #[rustfmt::skip]
    const TEST_SIGNED_IMAGE: &[u8] = &[
        // Image
        0xaa, 0xbb,
        // Magic string inverted
        0xb7, 0xac, 0x9c, 0xc8, 0x9c, 0xcd, 0x8f, 0x8b,
        0x86, 0x9b, 0xa5, 0xb7, 0xcd, 0xae, 0x94, 0x8e,
        // Signature
        0xa5, 0xa8, 0xaf, 0x9c, 0xb5, 0x98, 0xb8, 0xcc, 0xb5, 0x8b, 0x91, 0xb5, 0xc9, 0xa9, 0x8a,
        0xbe, 0x49, 0xdb, 0xc3, 0x82, 0x37, 0xff, 0x13, 0x9a, 0x96, 0xb1, 0xb2, 0x37, 0x4a, 0x41,
        0x35, 0x36, 0xd4, 0xed, 0xc7, 0xdf, 0x00, 0x80, 0x54, 0xde, 0x95, 0xbe, 0xc5, 0x1b, 0xbb,
        0x89, 0xa9, 0x35, 0x03, 0x62, 0xb0, 0xef, 0x73, 0x1f, 0x32, 0x4a, 0x5e, 0x93, 0x8c, 0x78,
        0x4e, 0xf5, 0x6a, 0x3f, 0xf5, 0x8f, 0x99, 0xf6, 0x11, 0x67, 0xa6, 0xc2, 0x12, 0xc7, 0xf5,
        0xb3, 0x3b, 0xb0, 0x12, 0x8e,
    ];

    #[test]
    fn retrieving_signed_image_succeeds() {
        let mut flash = FakeFlash::new(Address(0));
        let bank =
            Bank { index: 1, size: 512, location: Address(0), bootable: false, is_golden: false };
        flash.write(Address(0), &TEST_SIGNED_IMAGE).unwrap();

        let image = image_at(&mut flash, bank).unwrap();
        assert_eq!(image.size, 2usize);
        assert_eq!(image.location, bank.location);
        assert_eq!(image.bootable, false);
        assert_eq!(image.is_golden(), false);
    }

    #[test]
    fn retrieving_broken_image_fails() {
        let mut flash = FakeFlash::new(Address(0));
        let bank =
            Bank { index: 1, size: 512, location: Address(0), bootable: false, is_golden: false };

        let mut image: [u8; 98] = TEST_SIGNED_IMAGE.try_into().unwrap();
        image[0] = 0xCC; // Corrupted image body;
        flash.write(Address(0), &image).unwrap();
        assert_eq!(Err(Error::SignatureInvalid), image_at(&mut flash, bank));

        let mut image: [u8; 98] = TEST_SIGNED_IMAGE.try_into().unwrap();
        image[3] = 0xCC; // Corrupted magic string
        flash.write(Address(0), &image).unwrap();
        assert_eq!(Err(Error::BankEmpty), image_at(&mut flash, bank));

        let mut image: [u8; 98] = TEST_SIGNED_IMAGE.try_into().unwrap();
        image[96] = 0xCC; // Corrupted signature
        flash.write(Address(0), &image).unwrap();
        assert_eq!(Err(Error::SignatureInvalid), image_at(&mut flash, bank));
    }
}
