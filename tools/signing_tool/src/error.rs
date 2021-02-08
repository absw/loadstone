use std::fmt::{self, Display, Formatter};

pub enum File {
    Key,
    Image,
}

impl Display for File {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        use File::*;
        match self {
            Key => write!(f, "key"),
            Image => write!(f, "image"),
        }
    }
}

pub enum Error {
    FileReadFailed(File),
    FileOpenFailed(File),
    FileWriteFailed(File),
    KeyParseFailed,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        use Error::*;
        match self {
            FileReadFailed(file) => write!(f, "Failed to read {} file.", file),
            FileOpenFailed(file) => write!(f, "Failed to open {} file.", file),
            FileWriteFailed(file) => write!(f, "Failed to write {} file.", file),
            KeyParseFailed => write!(f, "Failed to parse the private key."),
        }
    }
}
