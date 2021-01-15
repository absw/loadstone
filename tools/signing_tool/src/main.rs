extern crate clap;

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

    println!("{:?}", matches);
}
