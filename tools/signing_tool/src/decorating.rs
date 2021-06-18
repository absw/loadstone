use crate::{
    error::{self, Error},
    open_image,
};
use blue_hal::utilities::iterator::UntilSequence;
use std::io::{Read, Write};

/// This string identifies a golden image, and must precede the magic string.
const GOLDEN_STRING: &str = "XPIcbOUrpG";
/// This string, INVERTED BYTEWISE must terminate any valid image, before the signature.
///
/// Note: Why inverted? Because if we used it as-is, no code that includes this
/// constant could be used as a firmware image, as it contains the magic string
/// halfway through.
pub const MAGIC_STRING: &str = "HSc7c2ptydZH2QkqZWPcJgG3JtnJ6VuA";
pub fn magic_string_inverted() -> Vec<u8> { MAGIC_STRING.as_bytes().iter().map(|b| !b).collect() }

pub fn decorate_file(image_filename: &str, is_golden: bool) -> Result<(), Error> {
    let file = open_image(image_filename)?;
    if file
        .bytes()
        .map(|b| b.unwrap())
        .until_sequence(magic_string_inverted().as_slice())
        .contains_sequence()
    {
        return Err(Error::FileAlreadySigned(error::File::Image));
    }
    let mut file = open_image(image_filename)?;
    if is_golden {
        file.write(GOLDEN_STRING.as_bytes())
            .map_err(|_| Error::FileWriteFailed(error::File::Image))?;
        println!("Successfully appended golden string.");
    }
    file.write(magic_string_inverted().as_slice())
        .map_err(|_| Error::FileWriteFailed(error::File::Image))?;
    println!("Successfully appended magic string.");
    Ok(())
}
