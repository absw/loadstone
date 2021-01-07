//! Error types and methods for the Secure Bootloader project.

use crate::hal::serial::Write;
use ufmt::{uwrite, uwriteln};

/// Top level error type for the bootloader. Unlike the specific
/// module errors, this error contains textual descriptions of the
/// problem as it is meant to be directly reported through USART.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Error {
    /// Error caused by a low level peripheral driver
    DriverError(&'static str),
    /// Error caused by a faulty configuration
    ConfigurationError(&'static str),
    /// Error caused by a high level device driver
    DeviceError(&'static str),
    BankInvalid,
    NotEnoughData,
    BankEmpty,
    ImageTooBig,
    FlashCorrupted,
    CrcInvalid,
}

/// Exposes a report_unwrap() method that behaves like
/// unwrap(), but also reports any errors via serial before panicking.
pub trait ReportOnUnwrap<T, S: Write> {
    fn report_unwrap(self, serial: &mut S) -> T;
}

/// Exposes a report_unwrap() method that behaves like
/// unwrap(), but also reports any errors via serial before panicking.
pub trait ReportOnUnwrapWithPrefix<T, S: Write> {
    fn report_unwrap(self, prefix: &'static str, serial: &mut S) -> T;
}

impl<T, S: Write> ReportOnUnwrap<T, S> for Result<T, Error> {
    fn report_unwrap(self, serial: &mut S) -> T {
        match self {
            Ok(value) => value,
            Err(error) => {
                error.report(serial);
                panic!();
            }
        }
    }
}

impl<T, S: Write> ReportOnUnwrapWithPrefix<T, S> for Result<T, Error> {
    fn report_unwrap(self, prefix: &'static str, serial: &mut S) -> T {
        match self {
            Ok(value) => value,
            Err(error) => {
                uprint!(serial, "{}", prefix);
                error.report(serial);
                panic!();
            }
        }
    }
}

impl Error {
    /// Reports error via abstract serial device
    pub fn report<S: Write>(&self, serial: &mut S) {
        match self {
            Error::DriverError(text) => uwriteln!(serial, "[DriverError] -> {}", text),
            Error::ConfigurationError(text) => {
                uwriteln!(serial, "[ConfigurationError] -> {}", text)
            }
            Error::DeviceError(text) => uwriteln!(serial, "[DeviceError] -> {}", text),
            Error::ImageTooBig => uwriteln!(serial, "[LogicError] -> Firmware Image too big"),
            Error::BankInvalid => uwriteln!(
                serial,
                "[LogicError] -> Bank doesn't exist or is invalid in this context"
            ),
            Error::BankEmpty => {
                uwriteln!(serial, "[LogicError] -> Bank is empty (contains no firmware image)")
            }
            Error::FlashCorrupted => {
                uwriteln!(serial, "[LogicError] -> Flash memory is corrupted or outdated")
            }
            Error::CrcInvalid => uwriteln!(serial, "[LogicError] -> Image CRC is invalid"),
            Error::NotEnoughData => {
                uwriteln!(serial, "[TransferError] -> Not enough image data received")
            }
        }
        .ok()
        .unwrap();
    }
}

pub trait ConvertibleToBootloaderError {}
