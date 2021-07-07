use p256::ecdsa::{
    signature::{Signature, Signer},
    SigningKey,
};
use std::str::FromStr;
use crc::{crc32, Hasher32};

use crate::{
    error::{self, Error},
    open_image,
};
use std::{
    fs::File,
    io::{Read, Write},
};

fn read_file(file: &mut File) -> Result<Vec<u8>, Error> {
    let mut contents = Vec::new();
    match file.read_to_end(&mut contents) {
        Ok(_) => Ok(contents),
        Err(_) => Err(Error::FileReadFailed(error::File::Image)),
    }
}

pub fn read_key(mut file: File) -> Result<SigningKey, Error> {
    let mut string = String::new();
    file.read_to_string(&mut string).map_err(|_| Error::KeyParseFailed)?;
    SigningKey::from_str(string.as_str()).map_err(|_| Error::KeyParseFailed)
}

/// Reads the contents of `file` and signs it using P256 ECDSA/SHA256 with the key in `key_file`.
pub fn sign_file(image_filename: &str, key: SigningKey) -> Result<usize, Error> {
    let mut file = open_image(image_filename)?;
    let plaintext = read_file(&mut file)?;
    let signature = key.sign(&plaintext);
    let bytes_written =
        file.write(signature.as_bytes()).map_err(|_| Error::FileWriteFailed(error::File::Image))?;

    if bytes_written == signature.as_bytes().len() {
        Ok(bytes_written)
    } else {
        Err(Error::FileWriteFailed(error::File::Image))
    }
}

pub fn calculate_and_append_crc(image_filename: &str) -> Result<usize, Error> {
    let mut file = open_image(image_filename)?;
    let plaintext = read_file(&mut file)?;

    let mut digest = crc32::Digest::new(crc32::IEEE);
    digest.write(&plaintext);

    let bytes_written =
        file.write(&digest.sum32().to_le_bytes()).map_err(|_| Error::FileWriteFailed(error::File::Image))?;

    if bytes_written == core::mem::size_of::<u32>() {
        Ok(bytes_written)
    } else {
        Err(Error::FileWriteFailed(error::File::Image))
    }
}
