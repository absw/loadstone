//! Interface to a serial device.
//!
//! This interface is block-agnostic thanks to the **nb** crate. This
//! means it can be used in a blocking manner (through the block! macro)
//! or in a manner compatible with schedulers, RTOS, etc. See the **nb**
//! crate documentation for details.
#![macro_use]

use nb::{self, block};

pub trait ReadWrite: Read + Write {}
impl<T: Read + Write> ReadWrite for T {}

pub use ufmt::uWrite as Write;

use super::time::{Milliseconds, Now};

/// UART read half
pub trait Read {
    type Error: Copy + Clone;

    /// Reads a single byte
    fn read(&mut self) -> nb::Result<u8, Self::Error>;
    fn bytes(&mut self) -> ReadIterator<Self> { ReadIterator { reader: self, errored: false } }
}

pub trait TimeoutRead {
    type Error: Copy + Clone;
    type Clock: Now;

    /// Reads a single byte
    fn read<T: Copy + Into<Milliseconds>>(&mut self, timeout: T) -> Result<u8, Self::Error>;
    fn bytes<T: Copy + Into<Milliseconds>>(&mut self, timeout: T) -> TimeoutReadIterator<Self, T> {
        TimeoutReadIterator { reader: self, errored: false, timeout }
    }
}

pub struct ReadIterator<'a, R: Read + ?Sized> {
    reader: &'a mut R,
    errored: bool,
}

pub struct TimeoutReadIterator<'a, R: TimeoutRead + ?Sized, T: Copy + Into<Milliseconds>> {
    reader: &'a mut R,
    errored: bool,
    timeout: T,
}

impl<'a, R: Read + ?Sized> Iterator for ReadIterator<'a, R> {
    type Item = Result<u8, <R as Read>::Error>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.errored {
            None
        } else {
            match block!(self.reader.read()) {
                Ok(byte) => Some(Ok(byte)),
                Err(e) => {
                    self.errored = true;
                    Some(Err(e))
                }
            }
        }
    }
}

impl<'a, R: TimeoutRead + ?Sized, T: Copy + Into<Milliseconds>> Iterator for TimeoutReadIterator<'a, R, T> {
    type Item = Result<u8, <R as TimeoutRead>::Error>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.errored {
            None
        } else {
            match self.reader.read(self.timeout) {
                Ok(byte) => Some(Ok(byte)),
                Err(e) => {
                    self.errored = true;
                    Some(Err(e))
                }
            }
        }
    }
}

/// Carries on silently if uncapable of writing.
#[macro_export]
macro_rules! uprint {
    ($serial:expr, $($arg:tt)+) => {
        let _ = uwrite!($serial, $($arg)+ );
    };
}

/// Carries on silently if uncapable of writing.
#[macro_export]
macro_rules! uprintln {
    ($serial:expr, $($arg:tt)+) => {
        let _ = uwriteln!($serial, $($arg)+ );
    };
}

/// Panics if uncapable of writing.
#[macro_export]
macro_rules! critical_uprint {
    ($serial:expr, $($arg:tt)+) => {
        uprint!($serial, $($arg)+ ).ok().unwrap();
    };
}

/// Panics if uncapable of writing.
#[macro_export]
macro_rules! critical_uprintln {
    ($serial:expr, $($arg:tt)+) => {
        uprintln!($serial, $($arg)+ ).ok().unwrap();
    };
}

#[cfg(test)]
mod test {
    #[derive(Debug, Default)]
    struct MockUsart {
        pub mock_value_to_read: u8,
        pub write_record: Vec<u8>,
    }

    impl Write for MockUsart {
        type Error = ();

        fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
            for byte in s.as_bytes() {
                self.write_record.push(*byte);
            }
            Ok(())
        }
        fn write_char(&mut self, c: char) -> Result<(), Self::Error> {
            Ok(self.write_record.push(c as u8))
        }
    }

    impl Read for MockUsart {
        type Error = ();

        /// Reads a single word
        fn read(&mut self) -> nb::Result<u8, Self::Error> { Ok(self.mock_value_to_read) }
    }

    use super::*;
    use ufmt::{uwrite, uwriteln};

    #[test]
    fn uwrite_macro_writes_bytes_with_no_newline() {
        // Given
        let mut mock_usart = MockUsart::default();
        let arbitrary_message = "Hello world!";
        let arbitrary_message_as_bytes: Vec<u8> =
            arbitrary_message.as_bytes().iter().cloned().collect();

        // When
        uprint!(mock_usart, "{}", arbitrary_message);

        // Then
        assert_eq!(arbitrary_message_as_bytes, mock_usart.write_record);
    }

    #[test]
    fn uwriteln_macro_writes_bytes_with_newline() {
        // Given
        let mut mock_usart = MockUsart::default();
        let arbitrary_message = "Hello world with newline!";
        let newline = "\n";
        let mut expected_message: Vec<u8> = arbitrary_message.as_bytes().iter().cloned().collect();
        expected_message.append(&mut newline.as_bytes().iter().cloned().collect());

        // When
        uwriteln!(mock_usart, "{}", arbitrary_message).unwrap();

        // Then
        assert_eq!(expected_message, mock_usart.write_record);
    }
}
