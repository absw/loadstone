extern crate sha2;
use sha2::{Digest, Sha256};

use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
};

pub fn get_file_hash(file: &mut File) -> Option<Vec<u8>> {
    let filesize = file.seek(SeekFrom::End(0)).ok()?;
    file.seek(SeekFrom::Start(0)).ok()?;

    let mut buffer = vec!();
    let bytes_read = file.read_to_end(&mut buffer).ok()?;

    if bytes_read == (filesize as usize) {
        let mut hasher = Sha256::new();
        hasher.update(buffer);
        Some(hasher.finalize().to_vec())
    } else {
        None
    }
}