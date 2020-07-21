//! Generic Bootloader.
//!
//! This module contains all bootloader functionality, with
//! the exception of how to construct one. Construction is
//! handled by the `port` module as it depends on board
//! specific information.
use crate::{
    devices::cli::Cli,
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
    SRL: serial::ReadWrite,
    LED: led::Toggle,
{
    pub(crate) external_flash: EXTF,
    pub(crate) mcu_flash: MCUF,
    pub(crate) post_led: LED,
    pub(crate) cli: Cli<SRL>,
}

impl<EXTF, MCUF, SRL, LED> Bootloader<EXTF, MCUF, SRL, LED>
where
    EXTF: flash::ReadWrite,
    MCUF: flash::ReadWrite,
    SRL: serial::ReadWrite,
    LED: led::Toggle,
{
    pub fn power_on_self_test(&mut self) {
        let Self { external_flash, mcu_flash, post_led, cli } = self;
        let _guard = Guard::new(post_led, Toggle::on, Toggle::off);
        Self::post_test_flash_simple_rwc(external_flash)
            .report_unwrap("[External Flash] ", cli.serial());
        uprintln!(cli.serial(), "External flash ID verification and simple RW cycle passed");
        Self::post_test_flash_simple_rwc(mcu_flash).report_unwrap("[MCU Flash] ", cli.serial());
        uprintln!(cli.serial(), "MCU flash ID verification and simple RW cycle passed");
        Self::post_test_flash_complex_rwc(external_flash)
            .report_unwrap("[External Flash] ", cli.serial());
        uprintln!(cli.serial(), "External flash complex RW cycle passed");
        Self::post_test_flash_complex_rwc(mcu_flash).report_unwrap("[MCU Flash] ", cli.serial());
        uprintln!(cli.serial(), "MCU flash complex RW cycle passed");
    }

    pub fn run(mut self) -> ! {
        loop {
            self.cli.run()
        }
    }

    fn post_test_flash_simple_rwc<F>(flash: &mut F) -> Result<(), Error>
    where
        F: flash::ReadWrite,
    {
        let failure = Error::PostError("Flash Read Write cycle failed");
        let mut magic_number_buffer = [0u8; 1];
        let mut new_magic_number_buffer = [0u8; 1];
        let (write_start, _) = F::writable_range();
        let (read_start, _) = F::readable_range();
        block!(flash.read(read_start, &mut magic_number_buffer)).map_err(|_| failure)?;
        new_magic_number_buffer[0] = magic_number_buffer[0].wrapping_add(1);
        block!(flash.write(write_start, &mut new_magic_number_buffer)).map_err(|_| failure)?;
        block!(flash.read(read_start, &mut magic_number_buffer)).map_err(|_| failure)?;
        if magic_number_buffer != new_magic_number_buffer {
            Err(failure)
        } else {
            Ok(())
        }
    }

    fn post_test_flash_complex_rwc<F>(flash: &mut F) -> Result<(), Error>
    where
        F: flash::ReadWrite,
    {
        let failure = Error::PostError("Complex Flash Read Write cycle failed");
        let magic_word_buffer = [0xAAu8, 0xBBu8, 0xCCu8, 0xDDu8];
        let superset_byte_buffer = [0xFFu8];
        let expected_final_buffer = [0xFFu8, 0xBBu8, 0xCCu8, 0xDDu8];
        let (write_start, _) = F::writable_range();
        let (read_start, _) = F::readable_range();
        block!(flash.write(write_start, &magic_word_buffer)).map_err(|_| failure)?;
        block!(flash.write(write_start, &superset_byte_buffer)).map_err(|_| failure)?;
        let mut final_buffer = [0x00; 4];
        block!(flash.read(read_start, &mut final_buffer)).map_err(|_| failure)?;
        if expected_final_buffer != final_buffer {
            Err(failure)
        } else {
            Ok(())
        }
    }
}
