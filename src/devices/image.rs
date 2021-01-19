use crate::error::Error;
use blue_hal::{hal::flash::{self, UnportableDeserialize, UnportableSerialize}, utilities::{buffer::find_subsequence, iterator::UntilSequence, memory::Address}};
use core::{cmp::min, mem::size_of};
use crc::{crc32, Hasher32};
use nb::{self, block};

pub(crate) const TRANSFER_BUFFER_SIZE: usize = 2048usize;

/// This string must terminate any valid images, after CRC
const MAGIC_STRING: &str = "HSc7c2ptydZH2QkqZWPcJgG3JtnJ6VuA";

#[derive(Clone, Copy, Debug)]
pub struct Bank<A: Address> {
    pub index: u8,
    pub size: usize,
    pub location: A,
    pub bootable: bool,
}

/// Image descriptor
#[derive(Clone)]
pub struct Image<A: Address> {
    size: usize,
    location: A,
    bootable: bool,
}

pub fn image_at<A, F>(flash: &mut F, bank: Bank<A>) -> Result<Image<A>, Error>
where
    A: Address,
    F: flash::ReadWrite<Address = A>,
    Error: From<F::Error>,
{
    let bytes = flash.bytes(bank.location).until_sequence(MAGIC_STRING.as_bytes());
    let (size, digest) = bytes.fold((0, crc32::Digest::new(crc32::IEEE)), |(mut size, mut digest), byte| {
        size += 1;
        digest.write(&[byte]);
        (size, digest)
    });
    let calculated_crc = digest.sum32();

    let crc_size_bytes = 4usize;
    let crc_offset = size - crc_size_bytes;
    let mut crc_bytes = [0u8; 4];
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

    #[test]
    fn retrieving_image_from_flash() {}
}
