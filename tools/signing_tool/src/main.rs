extern crate clap;

use std::{
    fs::{File, OpenOptions},
    process,
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

    println!("{:?}, {:?}", image, key);
}
