//! Generic Bootloader.
//!
//! This module contains all bootloader functionality, with
//! the exception of how to construct one. Construction is
//! handled by the `port` module as it depends on board
//! specific information.
use crate::{
    error::Error,
    hal::{flash, led, serial},
    utilities::guard::Guard,
};
use led::Toggle;
use nb::block;

pub struct Bootloader<E, F, S, L>
where
    E: flash::ReadWrite,
    F: flash::ReadWrite,
    S: serial::Write<u8>,
    L: led::Toggle,
    // Errors associated to the flashes can be converted to
    // Bootloader errors for further display
    Error: From<<E as flash::Write>::Error>,
    Error: From<<E as flash::Read>::Error>,
    Error: From<<F as flash::Write>::Error>,
    Error: From<<F as flash::Read>::Error>,
{
    pub(crate) external_flash: E,
    pub(crate) mcu_flash: F,
    pub(crate) post_led: L,
    pub(crate) serial: S,
}

impl<E, F, S, L> Bootloader<E, F, S, L>
where
    E: flash::ReadWrite,
    F: flash::ReadWrite,
    S: serial::Write<u8>,
    L: led::Toggle,
    Error: From<<E as flash::Write>::Error>,
    Error: From<<E as flash::Read>::Error>,
    Error: From<<F as flash::Write>::Error>,
    Error: From<<F as flash::Read>::Error>,
{
    pub fn power_on_self_test(&mut self) -> Result<(), Error> {
        Guard::new(&mut self.post_led, Toggle::on, Toggle::off);
        uprintln!(self.serial, Self::post_test_external_flash(&mut self.external_flash)?);
        Ok(())
    }

    pub fn run(self) -> ! { loop {} }

    fn post_test_external_flash(flash: &mut E) -> Result<&'static str, Error> {
        let mut magic_number_buffer = [0u8; 1];
        let mut new_magic_number_buffer = [0u8; 1];
        let (write_start, _) = E::writable_range();
        let (read_start, _) = E::readable_range();
        block!(flash.read(read_start, &mut magic_number_buffer))?;
        new_magic_number_buffer[0] = magic_number_buffer[0].wrapping_add(1);
        block!(flash.write(write_start, &mut new_magic_number_buffer))?;
        block!(flash.read(read_start, &mut magic_number_buffer))?;
        if magic_number_buffer != new_magic_number_buffer {
            Err(Error::LogicError("Flash read-write-read cycle failed!"))
        } else {
            Ok("[POST] -> External Flash ID verification and RWR cycle passed")
        }
    }
}
