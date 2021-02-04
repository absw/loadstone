mod error;
mod signing;

use crate::{
    error::{self as e, Error},
    signing::sign_file,
};
use clap::clap_app;
use std::fs::{File, OpenOptions};

fn run(image: String, key: String) -> Result<usize, Error> {
    let image_file = OpenOptions::new()
        .read(true)
        .append(true)
        .open(image)
        .map_err(|_| Error::FileOpenFailed(e::File::Image))?;
    let key_file = File::open(key).map_err(|_| Error::FileOpenFailed(e::File::Key))?;
    sign_file(image_file, key_file)
}

fn main() -> Result<(), String> {
    let matches = clap_app!(app =>
        (name: env!("CARGO_PKG_NAME"))
        (version: env!("CARGO_PKG_VERSION"))
        (author: env!("CARGO_PKG_AUTHORS"))
        (about: env!("CARGO_PKG_DESCRIPTION"))
        (@arg image: +required "The firmware image to be signed.")
        (@arg private_key: +required "The PKCS8 private key used to sign the image.")
    )
    .get_matches();

    let image = matches.value_of("image").unwrap().to_owned();
    let private_key = matches.value_of("private_key").unwrap().to_owned();

    match run(image, private_key) {
        Ok(signature_size) => {
            println!("Successfully appended signature to image ({} bytes).", signature_size);
            Ok(())
        }
        Err(e) => Err(e.to_string()),
    }
}
