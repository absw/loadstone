use core::mem::size_of;
use crate::error::Error;

use super::*;
use blue_hal::{hal::flash, utilities::{iterator::UntilSequence, memory::Address}};
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

    impl From<FakeError> for Error {
        fn from(_: FakeError) -> Self { Error::DeviceError("Something fake happened") }
    }
}
