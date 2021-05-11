//! Firmware image manipulation and inspection utilities.
//!
//! This module offers tools to partition flash memory spaces
//! into image banks and scan those banks for valid, signed images.

#[cfg(feature = "ecdsa-verify")]
pub mod image_ecdsa;

#[cfg(feature = "ecdsa-verify")]
pub use image_ecdsa::image_at as image_at;
#[cfg(feature = "ecdsa-verify")]
use image_ecdsa::*;

#[cfg(not(feature = "ecdsa-verify"))]
pub mod image_crc;
#[cfg(not(feature = "ecdsa-verify"))]
pub use image_crc::image_at as image_at;

use blue_hal::utilities::{buffer::CollectSlice, memory::Address};

/// This string precedes the CRC/Signature for golden images only
pub const GOLDEN_STRING: &str = "XPIcbOUrpG";

/// This string, INVERTED BYTEWISE must terminate any valid images, after CRC/Signature
///
/// Note: Why inverted? Because if we used it as-is, no code that includes this
/// constant could be used as a firmware image, as it contains the magic string
/// halfway through.
pub const MAGIC_STRING: &str = "HSc7c2ptydZH2QkqZWPcJgG3JtnJ6VuA";

/// utility function to invert the [`MAGIC_STRING`].
pub fn magic_string_inverted() -> [u8; MAGIC_STRING.len()] {
    let mut inverted = [0u8; MAGIC_STRING.len()];
    let mut bytes = MAGIC_STRING.as_bytes().iter().map(|b| !b);
    bytes.collect_slice(&mut inverted);
    inverted
}

/// Image bank descriptor.
///
/// A bank represents a section of flash memory that may contain a single signed
/// firmware image, for the purposes of booting, backup, update or recovery.
#[derive(Clone, Copy, Debug)]
pub struct Bank<A: Address> {
    /// Numeric identifier of the bank, unique even across multiple flash chips.
    pub index: u8,
    /// Size in bytes of the flash range occupied by this bank.
    pub size: usize,
    /// Address of the start of the image bank.
    pub location: A,
    /// Whether Loadstone is allowed to boot an image residing in this bank.
    pub bootable: bool,
    /// Whether this bank is allowed to supply golden images during the recovery
    /// process.
    ///
    /// NOTE: This field being `true` does not mean the bank is limited to *only*
    /// storing golden images. It is still able to store non-golden images, just like
    /// non-golden banks can store golden images. This is important to maintain
    /// the flash storage flexible and support different application requirements.
    ///
    /// The only enforced limitation is that, for an image to behave as a last
    /// resort fallback, both the bank and the image itself *must* be golden.
    pub is_golden: bool,
}

/// Image descriptor.
///
/// An image descriptor can only be constructed by scanning the flash and finding
/// a correctly decorated and signed firmware image.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Image<A: Address> {
    size: usize,
    location: A,
    bootable: bool,
    golden: bool,
    #[cfg(feature = "ecdsa-verify")]
    signature: image_ecdsa::Signature,
    #[cfg(not(feature = "ecdsa-verify"))]
    crc: u32,
}

impl<A: Address> Image<A> {
    /// Address of the start of the firmware image. Will generally coincide
    /// with the start of its associated image bank.
    pub fn location(&self) -> A { self.location }
    /// Size of the firmware image, excluding decoration and signature.
    pub fn size(&self) -> usize { self.size }
    /// Size of the firmware image, including decoration and signature.
    #[cfg(feature = "ecdsa-verify")]
    pub fn total_size(&self) -> usize {
        self.size()
            + SignatureSize::<NistP256>::to_usize()
            + MAGIC_STRING.len()
            + if self.is_golden() { GOLDEN_STRING.len() } else { 0 }
    }

    /// Size of the firmware image, including decoration and crc.
    #[cfg(not(feature = "ecdsa-verify"))]
    pub fn total_size(&self) -> usize {
        self.size()
            + core::mem::size_of::<u32>()
            + MAGIC_STRING.len()
            + if self.is_golden() { GOLDEN_STRING.len() } else { 0 }
    }
    /// Whether the image is verified to be golden (contains a golden string).
    /// A golden image is a high reliability, 'blessed' image able
    /// to be used as a last resort fallback.
    pub fn is_golden(&self) -> bool { self.golden }
    #[cfg(feature = "ecdsa-verify")]
    /// ECDSA signature of the firmware image. This is also used as an unique
    /// identifier for the firmware image for the purposes of updating.
    pub fn identifier(&self) -> image_ecdsa::Signature { self.signature }
    #[cfg(not(feature = "ecdsa-verify"))]
    /// Firmware image CRC. This is also used as an unique
    /// identifier for the firmware image for the purposes of updating.
    pub fn identifier(&self) -> u32 { self.crc }
}
