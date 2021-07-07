mod error;
mod signing;
mod decorating;

use crate::{
    decorating::decorate_file,
    error::{self as e, Error},
    signing::sign_file,
};
use clap::clap_app;
use signing::calculate_and_append_crc;
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
    private_key_filename: Option<String>,
    image_is_golden: bool,
) -> Result<usize, Error> {
    decorate_file(&image_filename, image_is_golden)?;

    if let Some(private_key_filename) = private_key_filename {
        let key_file =
            File::open(private_key_filename).map_err(|_| Error::FileOpenFailed(e::File::Key))?;
        let key = signing::read_key(key_file)?;
        sign_file(&image_filename, key)
    } else {
        calculate_and_append_crc(&image_filename)
    }
}

fn main() -> Result<(), String> {
    let matches = clap_app!(app =>
        (name: env!("CARGO_PKG_NAME"))
        (version: env!("CARGO_PKG_VERSION"))
        (author: env!("CARGO_PKG_AUTHORS"))
        (about: env!("CARGO_PKG_DESCRIPTION"))
        (@arg image: +required "The firmware image to be signed.")
        (@arg golden: -g --golden "Label the image as golden (Loadstone firmware fallback)")
        (@arg private_key: "The PKCS8 private key used to sign the image. \
            If absent, an IEEE CRC32 code will be appended instead of a signature.")
    )
    .get_matches();

    let image_filename = matches.value_of("image").unwrap().to_owned();
    let private_key_filename = matches.value_of("private_key").map(str::to_owned);

    match process_image_file(
        image_filename,
        private_key_filename.clone(),
        matches.occurrences_of("golden") > 0,
    ) {
        Ok(written_size) => {
            println!("Successfully appended {} to image ({} bytes).", if
                     private_key_filename.is_some() { "signature " } else { "CRC" },
                     written_size);
            Ok(())
        }
        Err(e) => Err(e.to_string()),
    }
}
