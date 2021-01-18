use ring::{
    signature::{RSA_PKCS1_SHA256, RsaKeyPair},
    rand::SystemRandom,
};
use std::{
    fs::File,
    io::{Read, Write},
};

fn read_file(file: &mut File) -> Result<Vec<u8>, String> {
    let mut contents = Vec::new();
    match file.read_to_end(&mut contents) {
        Ok(_) => Ok(contents),
        Err(_) => Err(String::from("Failed to read image file.")),
    }
}

fn read_key(mut file: File) -> Result<Vec<u8>, String> {
    let mut string = String::new();
    file.read_to_string(&mut string)
        .map_err(|_| String::from("Failed to read key file."))?;
    let encoded = string.lines()
        .filter(|l| !l.starts_with("-"))
        .fold(Vec::<u8>::new(), |mut data, line| {
            data.extend_from_slice(line.as_bytes());
            data
        });
    base64::decode(encoded)
        .map_err(|e| format!("Failed to decode key file: {}.", e))
}

pub fn create_zeroed_u8_vec(size: usize) -> Vec<u8> {
    let mut vector = Vec::<u8>::with_capacity(size);
    for _ in 0..size {
        vector.push(0);
    }
    vector
}

/// Reads the contents of `file` and signs it using RSA/SHA256 with the key in `key_file`.
/// NOTE: This assumes that `file` is in read/append mode and the key is PKCS1.
pub fn sign_file(mut file: File, key_file: File) -> Result<(), String> {
    let raw_key = read_key(key_file)?;
    let plaintext = read_file(&mut file)?;
    let key = RsaKeyPair::from_pkcs8(&raw_key)
        .map_err(|e| format!("Failed to parse key: {}.", e))?;

    let rng = SystemRandom::new();
    let mut signature = create_zeroed_u8_vec(key.public_modulus_len());

    key.sign(&RSA_PKCS1_SHA256, &rng, &plaintext, &mut signature)
        .map_err(|_| String::from("Failed to generate image signature."))?;

    let bytes_written = file.write(&signature)
        .map_err(|e| format!("Failed to write signature to file: {}.", e))?;

    if bytes_written == signature.len() {
        Ok(())
    } else {
        Err(String::from("Failed to write entire signature to file."))
    }
}