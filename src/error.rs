//! Error types and methods for the Secure Bootloader project.

use crate::hal::serial::Write;

/// Top level error type for the bootloader. Unlike the specific
/// module errors, this error contains textual descriptions of the
/// problem as it is meant to be directly reported through USART.
#[derive(Debug, Copy, Clone)]
pub enum Error {
    /// Error caused by a low level peripheral driver
    DriverError(&'static str),
    /// Error caused by a faulty configuration
    ConfigurationError(&'static str),
    /// Error caused by a high level device driver
    DeviceError(&'static str),
    /// Error caused by faulty business logic
    LogicError(&'static str),
}

/// Exposes a report_unwrap() method that behaves like
/// unwrap(), but also reports any errors via serial before panicking.
pub trait ReportOnUnwrap<T, S: Write<u8>> {
    fn report_unwrap(self, serial: &mut S) -> T;
}

impl<T, S: Write<u8>> ReportOnUnwrap<T, S> for Result<T, Error> {
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

impl Error {
    /// Reports error via abstract serial device
    pub fn report<S: Write<u8>>(&self, serial: &mut S) {
        match self {
            Error::DriverError(text) => {
                uprint!(serial, "[DriverError] -> ");
                uprintln!(serial, text);
            }
            Error::ConfigurationError(text) => {
                uprint!(serial, "[ConfigurationError] -> ");
                uprintln!(serial, text);
            }
            Error::DeviceError(text) => {
                uprint!(serial, "[DeviceError] -> ");
                uprintln!(serial, text);
            }
            Error::LogicError(text) => {
                uprint!(serial, "[LogicError] -> ");
                uprintln!(serial, text);
            }
        };
    }
}
