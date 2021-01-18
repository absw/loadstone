mod signing;

use crate::signing::*;
use clap::{clap_app};
use std::{
    fs::{File, OpenOptions},
    process,
};

fn run_with_files(image: File, key: File) -> Result<String, String> {
    sign_file(image, key)
        .map(|()| String::from("Successfully appended signature to image."))
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