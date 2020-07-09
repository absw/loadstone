//! Error type for the Secure Bootloader project as a whole.
use crate::hal::serial::Write;

#[derive(Debug, Copy, Clone)]
pub enum Error {
    DriverError(&'static str),
    ConfigurationError(&'static str),
    DeviceError(&'static str),
    LogicError(&'static str),
}

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
    pub fn report<S: Write<u8>>(&self, serial: &mut S) {
        match self {
            Error::DriverError(text) => {
                uprint!(serial, "[DriverError] -> ");
                uprintln!(serial, text);
            },
            Error::ConfigurationError(text) => {
                uprint!(serial, "[ConfigurationError] -> ");
                uprintln!(serial, text);
            },
            Error::DeviceError(text) => {
                uprint!(serial, "[DeviceError] -> ");
                uprintln!(serial, text);
            },
            Error::LogicError(text) => {
                uprint!(serial, "[LogicError] -> ");
                uprintln!(serial, text);
            },
        };
    }
}
