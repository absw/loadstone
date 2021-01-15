extern crate sha2;
use sha2::{Digest, Sha256};

use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
};

pub fn get_file_hash(file: &mut File) -> Vec<u8> {
    // TODO: Consider reading file part-wise;
    let _ = file.seek(SeekFrom::Start(0));
    let mut buffer = vec!();
    let _ = file.read_to_end(&mut buffer);
    let mut hasher = Sha256::new();
    hasher.update(buffer);
    hasher.finalize().to_vec()
}