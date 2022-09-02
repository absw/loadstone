//! Loadstone Error types and methods

use blue_hal::{hal::serial::Write, uprint};
use defmt::Format;
use ufmt::{uwrite, uwriteln};

/// Top level error type for the bootloader. Unlike the specific
/// module errors, this error contains textual descriptions of the
/// problem as it is meant to be directly reported through USART.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Format)]
pub enum Error {
    /// Error caused by a low level peripheral driver
    DriverError(&'static str),
    /// Error caused by a faulty configuration
    ConfigurationError(&'static str),
    /// Error caused by a high level device driver
    DeviceError(&'static str),
    BankInvalid,
    BankEmpty,
    ImageTooBig,
    ImageIsNotGolden,
    NoGoldenBankSupport,
    FlashCorrupted,
    NoExternalFlash,
    NoImageToRestoreFrom,
    NoRecoverySupport,
    SignatureInvalid,
    CrcInvalid,
}

pub trait Convertible {
    fn into(self) -> Error;
}
impl<T: Convertible> From<T> for Error {
    fn from(t: T) -> Self {
        t.into()
    }
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
            Error::DriverError(text) => uwriteln!(serial, "[Driver Error] -> {}", text),
            Error::ConfigurationError(text) => {
                uwriteln!(serial, "[Configuration Error] -> {}", text)
            }
            Error::DeviceError(text) => uwriteln!(serial, "[Device Error] -> {}", text),
            Error::ImageTooBig => uwriteln!(serial, "[Logic Error] -> Firmware image too big"),
            Error::BankInvalid => uwriteln!(
                serial,
                "[Logic Error] -> Bank doesn't exist or is invalid in this context"
            ),
            Error::BankEmpty => {
                uwriteln!(
                    serial,
                    "[Logic Error] -> Bank is empty (contains no firmware image)"
                )
            }
            Error::FlashCorrupted => {
                uwriteln!(
                    serial,
                    "[Logic Error] -> Flash memory is corrupted or outdated"
                )
            }
            Error::SignatureInvalid => {
                uwriteln!(serial, "[LogicError] -> Image signature is invalid")
            }
            Error::NoImageToRestoreFrom => {
                uwriteln!(serial, "[Logic Error] -> No image to restore from")
            }
            Error::NoExternalFlash => {
                uwriteln!(
                    serial,
                    "[Logic Error] -> No external flash in this configuration"
                )
            }
            Error::ImageIsNotGolden => {
                uwriteln!(serial, "[Logic Error] -> Image is not golden")
            }
            Error::NoGoldenBankSupport => {
                uwriteln!(serial, "[Logic Error] -> No golden bank support")
            }
            Error::NoRecoverySupport => {
                uwriteln!(serial, "[Logic Error] -> No image recovery support")
            }
            Error::CrcInvalid => {
                uwriteln!(serial, "[Logic Error] -> Image CRC is invalid")
            }
        }
        .ok()
        .unwrap();
    }
}
