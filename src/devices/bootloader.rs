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

pub struct Bootloader<EXTF, MCUF, SRL, LED>
where
    EXTF: flash::ReadWrite,
    MCUF: flash::ReadWrite,
    SRL: serial::Write,
    LED: led::Toggle,
{
    pub(crate) external_flash: EXTF,
    pub(crate) mcu_flash: MCUF,
    pub(crate) post_led: LED,
    pub(crate) serial: SRL,
}

impl<EXTF, MCUF, SRL, LED> Bootloader<EXTF, MCUF, SRL, LED>
where
    EXTF: flash::ReadWrite,
    MCUF: flash::ReadWrite,
    SRL: serial::Write,
    LED: led::Toggle,
{
    pub fn power_on_self_test(&mut self) {
        let Self { external_flash, mcu_flash, post_led, serial } = self;
        let _guard = Guard::new(post_led, Toggle::on, Toggle::off);
        Self::post_test_flash(external_flash) .report_unwrap("[External Flash] ", serial);
        uprintln!(serial, "External flash ID verification and RWR cycle passed");
        Self::post_test_flash(mcu_flash).report_unwrap("[Mcu Flash] ", serial);
        uprintln!(serial, "Mcu flash ID verification and RWR cycle passed");
    }

    pub fn run(self) -> ! { loop {} }

    fn post_test_flash<F>(flash: &mut F) -> Result<(), Error>
    where
        F: flash::ReadWrite,
    {
        let post_failed = Error::PostError("Flash Read Write cycle failed");
        let mut magic_number_buffer = [0u8; 1];
        let mut new_magic_number_buffer = [0u8; 1];
        let (write_start, _) = F::writable_range();
        let (read_start, _) = F::readable_range();
        block!(flash.read(read_start, &mut magic_number_buffer)).map_err(|_| post_failed)?;
        new_magic_number_buffer[0] = magic_number_buffer[0].wrapping_add(1);
        block!(flash.write(write_start, &mut new_magic_number_buffer)).map_err(|_| post_failed)?;
        block!(flash.read(read_start, &mut magic_number_buffer)).map_err(|_| post_failed)?;
        if magic_number_buffer != new_magic_number_buffer {
            Err(Error::PostError("Flash Read Write cycle failed"))
        } else {
            Ok(())
        }
    }
}
