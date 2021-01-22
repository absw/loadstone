use crate::error::{self, Error};
use ring::{
    rand::SystemRandom,
    signature::{RsaKeyPair, RSA_PKCS1_SHA256},
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

fn read_key(mut file: File) -> Result<Vec<u8>, Error> {
    let mut string = String::new();
    file.read_to_string(&mut string)
        .map_err(|_| Error::KeyParseFailed)?;
    let encoded =
        string
            .lines()
            .filter(|l| !l.starts_with("-"))
            .fold(Vec::<u8>::new(), |mut data, line| {
                data.extend_from_slice(line.as_bytes());
                data
            });
    base64::decode(encoded).map_err(|_| Error::KeyParseFailed)
}

/// Reads the contents of `file` and signs it using RSA/SHA256 with the key in `key_file`.
/// NOTE: This assumes that `file` is in read/append mode and the key is PKCS8.
pub fn sign_file(mut file: File, key_file: File) -> Result<(), Error> {
    let raw_key = read_key(key_file)?;
    let plaintext = read_file(&mut file)?;
    let key =
        RsaKeyPair::from_pkcs8(&raw_key).map_err(|_| Error::KeyParseFailed)?;

    let rng = SystemRandom::new();
    let mut signature = vec![0u8; key.public_modulus_len()];

    key.sign(&RSA_PKCS1_SHA256, &rng, &plaintext, &mut signature)
        .map_err(|_| Error::SignatureGenerationFailed)?;

    let bytes_written = file
        .write(&signature)
        .map_err(|_| Error::FileWriteFailed(error::File::Image))?;

    if bytes_written == signature.len() {
        Ok(())
    } else {
        Err(Error::FileWriteFailed(error::File::Image))
    }
}
