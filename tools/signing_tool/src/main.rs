mod hashing;
use crate::hashing::*;

mod signing;
use crate::signing::*;

extern crate clap;
use clap::{clap_app};

extern crate base64;

use std::{
    fs::{File, OpenOptions},
    process,
    io::{Read, Write},
};

fn read_key(mut file: File) -> Option<Vec<u8>> {
    let mut string = String::new();
    file.read_to_string(&mut string)
        .ok()?;
    let encoded = string.lines()
        .filter(|l| !l.starts_with("-"))
        .fold(Vec::<u8>::new(), |mut data, line| {
            data.extend_from_slice(line.as_bytes());
            data
        });
    base64::decode(encoded)
        .ok()
}

fn run_with_files(mut image: File, key: File) -> Result<String, String> {
    let hash = get_file_hash(&mut image)
        .ok_or(String::from("Failed to calculate image hash"))?;
    let key = read_key(key)
        .ok_or(String::from("Failed to read private key"))?;
    let signature = sign(&hash, &key)?;
    let bytes_written = image.write(&signature)
        .map_err(|e| {
            format!("Failed to append signature to image: {}", e)
        })?;
    if bytes_written == signature.len() {
        Ok(String::from("Successfully appended signature to image."))
    } else {
        Err(String::from("Error: signature only partially written."))
    }
}

fn run_with_file_names(image: String, key: String) -> Result<String, String> {
    let image_file = OpenOptions::new()
        .read(true)
        .append(true)
        .open(image)
        .map_err(|e| {
            format!("Failed to open image file: {}", e)
        })?;
    let key_file = File::open(key)
        .map_err(|e| {
            format!("Failed to open key file: {}", e)
        })?;
    run_with_files(image_file, key_file)
}

fn main() {
    let matches = clap_app!(app =>
        (name: env!("CARGO_PKG_NAME"))
        (version: env!("CARGO_PKG_VERSION"))
        (author: env!("CARGO_PKG_AUTHORS"))
        (about: env!("CARGO_PKG_DESCRIPTION"))
        (@arg image: +required "The firmware image to be signed.")
        (@arg private_key: +required "The private key used to sign the image.")
    ).get_matches();

    let image = matches.value_of("image")
        .unwrap()
        .to_owned();
    let private_key = matches.value_of("private_key")
        .unwrap()
        .to_owned();

    match run_with_file_names(image, private_key) {
        Ok(s) => {
            println!("{}", s);
        },
        Err(s) => {
            eprintln!("{}", s);
            process::exit(1);
        }
    }
}