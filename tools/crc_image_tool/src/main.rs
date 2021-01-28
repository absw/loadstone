#![feature(with_options)]
use clap::Clap;
use crc::crc32::{self, Hasher32};
use std::{io::BufReader, io::BufRead, fs::File, io::prelude::*};
use byteorder::{LittleEndian, WriteBytesExt};

const MAGIC_STRING: &str = "HSc7c2ptydZH2QkqZWPcJgG3JtnJ6VuA";
const GOLDEN_STRING: &str = "XPIcbOUrpG";

#[derive(Clap)]
#[clap(about = "Tool to calculate and append CRC to firmware images", version = "1.0", author = "Pablo Mansanet <pablo.mansanet@bluefruit.co.uk>")]
struct Opts {
    #[clap(about = "Filename to append CRC to")]
    filename: String,
    #[clap(short, about = "Label the image as golden (Loadstone firmware fallback)")]
    golden: bool,
}

fn main() -> std::io::Result<()> {
    let opts = Opts::parse();
    let mut digest = crc32::Digest::new(crc32::IEEE);

    println!("Calculating CRC32/IEEE for {}", &opts.filename);
    {
        let firmware = File::with_options().read(true).open(&opts.filename)?;
        let mut buf_reader = BufReader::new(firmware);

        while buf_reader.fill_buf()?.len() > 0 {
            digest.write(buf_reader.buffer());
            buf_reader.consume(buf_reader.buffer().len())
        }
    }
    digest.write(GOLDEN_STRING.as_bytes());
    println!("Final CRC is {} (0x{:8x})", digest.sum32(), digest.sum32());

    let mut final_crc = [0u8; 4];
    (&mut final_crc[..]).write_u32::<LittleEndian>(digest.sum32())?;

    let mut firmware = File::with_options().append(true).open(&opts.filename)?;
    println!("Appending metadata to the end of {}", &opts.filename);
    if opts.golden {
        println!("* Appending golden image string");
        firmware.write(GOLDEN_STRING.as_bytes())?;
    }

    println!("* Appending CRC string");
    firmware.write(&final_crc)?;
    println!("* Appending magic string");
    firmware.write(MAGIC_STRING.as_bytes())?;

    println!("Done!");
    Ok(())
}
