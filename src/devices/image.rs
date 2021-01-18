use crate::error::Error;
use blue_hal::{
    hal::flash::{self, UnportableDeserialize, UnportableSerialize},
    utilities::memory::Address,
};
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
        unimplemented!();
        //if size <= size_of::<u32>() {
        //    return Err(Error::BankEmpty);
        //}

        //let mut buffer = [0u8; TRANSFER_BUFFER_SIZE];
        //let mut byte_index = 0usize;
        //let mut digest = crc32::Digest::new(crc32::IEEE);
        //let size_before_crc = size.saturating_sub(size_of::<u32>());

        //while byte_index < size_before_crc {
        //    let remaining_size = size_before_crc.saturating_sub(byte_index);
        //    let bytes_to_read = min(TRANSFER_BUFFER_SIZE, remaining_size);
        //    let slice = &mut buffer[0..bytes_to_read];
        //    block!(flash.read(location + byte_index, slice))?;
        //    digest.write(slice);
        //    byte_index += bytes_to_read;
        //}

        //let mut crc_bytes = [0u8; 4];
        //block!(flash.read(location + byte_index, &mut crc_bytes))?;
        //let crc = u32::from_le_bytes(crc_bytes);
        //let calculated_crc = digest.sum32();
        //if crc == calculated_crc {
        //    Ok(crc)
        //} else {
        //    Err(Error::CrcInvalid)
        //}
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
    fn retrieving_image_from_flash() {
    }
}
