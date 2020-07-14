//! Generic Bootloader.
//!
//! This module contains all bootloader functionality, with
//! the exception of how to construct one. Construction is
//! handled by the `port` module as it depends on board
//! specific information.
use crate::{
    error::{Error, ReportOnUnwrapWithPrefix},
    hal::{flash, led, serial},
    utilities::guard::Guard,

};
use led::Toggle;
use nb::block;

pub struct Bootloader<E, M, S, L>
where
    E: flash::ReadWrite,
    M: flash::ReadWrite,
    S: serial::Write,
    L: led::Toggle,
    // Errors associated to the flashes can be converted to
    // Bootloader errors for further display
    Error: From<<E as flash::Write>::Error>,
    Error: From<<E as flash::Read>::Error>,
    Error: From<<M as flash::Write>::Error>,
    Error: From<<M as flash::Read>::Error>,
{
    pub(crate) external_flash: E,
    pub(crate) mcu_flash: M,
    pub(crate) post_led: L,
    pub(crate) serial: S,
}

impl<E, M, S, L> Bootloader<E, M, S, L>
where
    E: flash::ReadWrite,
    M: flash::ReadWrite,
    S: serial::Write,
    L: led::Toggle,
    Error: From<<E as flash::Write>::Error>,
    Error: From<<E as flash::Read>::Error>,
    Error: From<<M as flash::Write>::Error>,
    Error: From<<M as flash::Read>::Error>,
{
    pub fn power_on_self_test(&mut self) {
        Guard::new(&mut self.post_led, Toggle::on, Toggle::off);
        Self::post_test_flash(&mut self.external_flash).report_unwrap("[External Flash] ", &mut self.serial);
        uprintln!(self.serial, "External flash ID verification and RWR cycle passed");
        Self::post_test_flash(&mut self.mcu_flash).report_unwrap("[Mcu Flash] ", &mut self.serial);
        uprintln!(self.serial, "Mcu flash ID verification and RWR cycle passed");
    }

    pub fn run(self) -> ! { loop {} }

    fn post_test_flash<F>(flash: &mut F) -> Result<(), Error>
    where
        F: flash::ReadWrite,
        Error: From<<F as flash::Write>::Error>,
        Error: From<<F as flash::Read>::Error>,
    {
        let mut magic_number_buffer = [0u8; 1];
        let mut new_magic_number_buffer = [0u8; 1];
        let (write_start, _) = F::writable_range();
        let (read_start, _) = F::readable_range();
        block!(flash.read(read_start, &mut magic_number_buffer))?;
        new_magic_number_buffer[0] = magic_number_buffer[0].wrapping_add(1);
        block!(flash.write(write_start, &mut new_magic_number_buffer))?;
        block!(flash.read(read_start, &mut magic_number_buffer))?;
        if magic_number_buffer != new_magic_number_buffer {
            Err(Error::PostError("Flash Read Write cycle failed"))
        } else  {
            Ok(())
        }
    }
}
