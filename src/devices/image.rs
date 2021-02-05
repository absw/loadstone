use ::ecdsa::{generic_array::typenum::Unsigned, SignatureSize};
use blue_hal::{
    hal::flash,
    utilities::{buffer::CollectSlice, iterator::UntilSequence, memory::Address},
};
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
    // Magic string is part of the digest
    digest.update(&magic_string_inverted());

    let signature_position = bank.location + image_size + MAGIC_STRING.len();
    let signature_bytes = &mut buffer[0..SignatureSize::<NistP256>::to_usize()];
    block!(flash.read(signature_position, signature_bytes))?;

    let signature = Signature::from_asn1(signature_bytes).map_err(|_| Error::SignatureInvalid)?;
    key.verify_digest(digest, &signature).map_err(|_| Error::SignatureInvalid)?;

    let golden_string_position = bank.location + image_size.saturating_sub(GOLDEN_STRING.len());
    let golden_bytes = &mut buffer[0..GOLDEN_STRING.len()];
    block!(flash.read(golden_string_position, golden_bytes))?;
    let golden = golden_bytes == GOLDEN_STRING.as_bytes();

    if golden { image_size = image_size.saturating_sub(GOLDEN_STRING.len()); }

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

    const TEST_SIGNED_IMAGE: [u8; 105] = [0xaa, 0xbb, 0xb7, 0xac, 0x9c, 0xc8, 0x9c, 0xcd, 0x8f, 0x8b,
          0x86, 0x9b, 0xa5, 0xb7, 0xcd, 0xae, 0x94, 0x8e, 0xa5, 0xa8,
          0xaf, 0x9c, 0xb5, 0x98, 0xb8, 0xcc, 0xb5, 0x8b, 0x91, 0xb5,
          0xc9, 0xa9, 0x8a, 0xbe, 0x30, 0x45, 0x02, 0x20, 0x0c, 0x5f,
          0xc4, 0x7f, 0x5f, 0xfe, 0x75, 0xab, 0x14, 0x23, 0x51, 0x64,
          0xf2, 0x3e, 0x33, 0x5e, 0x47, 0xdc, 0x64, 0xf9, 0x63, 0xaa,
          0x49, 0x9a, 0x59, 0x45, 0xf2, 0x7e, 0xc4, 0x1b, 0x4a, 0x93,
          0x02, 0x21, 0x00, 0xa2, 0xf6, 0x3c, 0x51, 0xe4, 0xcb, 0x7c,
          0x22, 0x2b, 0x94, 0x45, 0xad, 0x12, 0x47, 0x8f, 0x8d, 0x21,
          0xa7, 0x49, 0x31, 0x3e, 0x3f, 0xb7, 0xa1, 0x41, 0x48, 0x3a,
          0xcc, 0x42, 0x33, 0x54, 0x30];

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

    //#[test]
    //fn retrieving_broken_image_fails() {
    //    let mut flash = FakeFlash::new(Address(0));
    //    let bank =
    //        Bank { index: 1, size: 512, location: Address(0), bootable: false, is_golden: false };
    //    let mut image_with_crc = test_image_with_crc();
    //    image_with_crc[1] = 0xFF; // This will corrupt the image, making the CRC obsolete
    //    flash.write(Address(0), &image_with_crc).unwrap();
    //    assert_eq!(Err(Error::CrcInvalid), image_at(&mut flash, bank));

    //    let bank =
    //        Bank { index: 1, size: 512, location: Address(0), bootable: false, is_golden: false };
    //    let mut image_with_crc = test_image_with_crc();
    //    image_with_crc[4] = 0xFF; // This will break the CRC directly
    //    flash.write(Address(0), &image_with_crc).unwrap();
    //    assert_eq!(Err(Error::CrcInvalid), image_at(&mut flash, bank));

    //    let bank =
    //        Bank { index: 1, size: 512, location: Address(0), bootable: false, is_golden: false };
    //    let mut image_with_crc = test_image_with_crc();
    //    image_with_crc[12] = 0xFF; // The magic string is not present to delineate the image
    //    flash.write(Address(0), &image_with_crc).unwrap();
    //    assert_eq!(Err(Error::CrcInvalid), image_at(&mut flash, bank));
    //}
}
