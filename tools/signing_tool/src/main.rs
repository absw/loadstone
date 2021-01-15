mod hashing;
use crate::hashing::*;

mod signing;
use crate::signing::*;

extern crate clap;
extern crate base64;

use std::{
    fs::{File, OpenOptions},
    process,
    io::Read,
};

fn open_file(path: &str, append: bool) -> Option<File> {
    let file = OpenOptions::new()
        .read(true)
        .append(append)
        .open(path);
    match file {
        Ok(f) => Some(f),
        Err(e) => {
            eprintln!("Failed to open '{}': {}.", path, e);
            None
        },
    }
}

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

fn main() {
    let matches = clap::App::new("Loadstone Image Signing Tool")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(clap::Arg::with_name("image")
            .index(1)
            .required(true)
            .help("The firmware image to be signed."))
        .arg(clap::Arg::with_name("private_key")
            .index(2)
            .required(true)
            .help("The private key used to sign the image."))
        .get_matches();

    let image_path = matches.value_of("image").unwrap();
    let key_path = matches.value_of("private_key").unwrap();
    let image = open_file(image_path, true);
    let key = open_file(key_path, false);

    if image.is_none() || key.is_none() {
        process::exit(1);
    }
    let mut image = image.unwrap();
    let key = key.unwrap();

    println!("{:?}, {:?}", image, key);

    let hash = get_file_hash(&mut image);
    println!("{:?}", hash);

    let raw_key = match read_key(key) {
        Some(k) => k,
        None => {
            eprintln!("Failed to decode private key.");
            process::exit(1);
        },
    };

    let signature = match sign(&hash[..], &raw_key[..]) {
        Ok(s) => s,
        Err(_) => { process::exit(1); },
    };

    println!("{:?}", signature);
}
