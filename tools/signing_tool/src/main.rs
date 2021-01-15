mod hashing;
use crate::hashing::*;

mod signing;
use crate::signing::*;

extern crate clap;
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
        .open(image);
    let key_file = File::open(key);

    match (image_file, key_file) {
        (Ok(i), Ok(k)) => {
            run_with_files(i, k)
        },
        (Err(i), Ok(_)) => {
            Err(format!("Failed to open image file: {}", i))
        },
        (Ok(_), Err(k)) => {
            Err(format!("Failed to open key file: {}", k))
        },
        (Err(i), Err(k)) => {
            Err(format!("Failed to open files for key ({}) and image ({}).\n", i, k))
        },
    }
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
