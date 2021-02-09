mod error;
mod signing;
mod decorating;

use crate::{
    decorating::decorate_file,
    error::{self as e, Error},
    signing::sign_file,
};
use clap::clap_app;
use std::fs::{File, OpenOptions};

fn open_image(filename: &str) -> Result<File, Error> {
    OpenOptions::new()
        .read(true)
        .append(true)
        .open(filename)
        .map_err(|_| Error::FileOpenFailed(e::File::Image))
}

fn process_image_file(
    image_filename: String,
    private_key_filename: String,
    image_is_golden: bool,
) -> Result<usize, Error> {
    let key_file =
        File::open(private_key_filename).map_err(|_| Error::FileOpenFailed(e::File::Key))?;
    let key = signing::read_key(key_file)?;
    decorate_file(&image_filename, image_is_golden)?;
    sign_file(&image_filename, key)
}

fn main() -> Result<(), String> {
    let matches = clap_app!(app =>
        (name: env!("CARGO_PKG_NAME"))
        (version: env!("CARGO_PKG_VERSION"))
        (author: env!("CARGO_PKG_AUTHORS"))
        (about: env!("CARGO_PKG_DESCRIPTION"))
        (@arg image: +required "The firmware image to be signed.")
        (@arg golden: -g --golden "Label the image as golden (Loadstone firmware fallback)")
        (@arg private_key: +required "The PKCS8 private key used to sign the image.")
    )
    .get_matches();

    let image_filename = matches.value_of("image").unwrap().to_owned();
    let private_key_filename = matches.value_of("private_key").unwrap().to_owned();

    match process_image_file(
        image_filename,
        private_key_filename,
        matches.occurrences_of("golden") > 0,
    ) {
        Ok(signature_size) => {
            println!("Successfully appended signature to image ({} bytes).", signature_size);
            Ok(())
        }
        Err(e) => Err(e.to_string()),
    }
}
