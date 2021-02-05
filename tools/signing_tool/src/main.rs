mod error;
mod signing;
mod decorating;

use crate::{
    error::{self as e, Error},
    signing::sign_file,
    decorating::decorate_file,
};
use clap::clap_app;
use std::fs::{File, OpenOptions};

fn process_image_file(image_filename: String, private_key_filename: String, image_is_golden: bool) -> Result<usize, Error> {
    let mut image_file = OpenOptions::new()
        .read(true)
        .append(true)
        .open(image_filename)
        .map_err(|_| Error::FileOpenFailed(e::File::Image))?;
    let key_file = File::open(private_key_filename).map_err(|_| Error::FileOpenFailed(e::File::Key))?;
    decorate_file(&mut image_file, image_is_golden)?;
    sign_file(image_file, key_file)
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

    match process_image_file(image_filename, private_key_filename, matches.occurrences_of("g") > 0) {
        Ok(signature_size) => {
            println!(
                "Successfully appended signature to image ({} bytes).",
                signature_size
            );
            Ok(())
        }
        Err(e) => Err(e.to_string()),
    }
}
