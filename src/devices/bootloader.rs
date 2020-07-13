//! Generic Bootloader.
//!
//! This module contains all bootloader functionality, with
//! the exception of how to construct one. Construction is
//! handled by the `port` module as it depends on board
//! specific information.
use core::convert::Into;
use crate::{
    error::Error,
    hal::{serial, flash, led},
};
use core::{fmt, marker::PhantomData};
use nb::block;

pub struct Bootloader<E, A, S, L>
where
    // E is some writable and readable external flash
    E: flash::Write<A> + flash::Read<A>,
    // A is some memory address for the external flash,
    // that can be copied, cloned and displayed for debug
    A: Copy + Clone + fmt::Debug,
    // S is some serial that can write bytes, for CLI and logging
    S: serial::Write<u8>,
    // L is some LED that can display POST progress
    L: led::Toggle,
    // Errors associated to the flash can be converted to Bootloader
    // errors for further display
    Error: From<<E as flash::Write<A>>::Error>,
    Error: From<<E as flash::Read<A>>::Error>,
{
    pub(crate) flash: E,
    pub(crate) post_led: L,
    pub(crate) serial: S,
    pub(crate) _marker: PhantomData<A>,
}

impl<E, A, S, L> Bootloader<E, A, S, L>
where
    E: flash::Write<A> + flash::Read<A>,
    A: Copy + Clone + fmt::Debug,
    S: serial::Write<u8>,
    L: led::Toggle,
    Error: From<<E as flash::Write<A>>::Error>,
    Error: From<<E as flash::Read<A>>::Error>,
{
    pub fn power_on_self_test(&mut self) -> Result<(), Error> {
        let mut magic_number_buffer = [0u8; 1];
        let mut new_magic_number_buffer = [0u8; 1];

        self.post_led.on();
        let (start, _) = E::writable_range();
        block!(self.flash.read(start, &mut magic_number_buffer))?;
        new_magic_number_buffer[0] = magic_number_buffer[0].wrapping_add(1);
        block!(self.flash.write(start, &mut new_magic_number_buffer))?;
        block!(self.flash.read(start, &mut magic_number_buffer))?;
        self.post_led.off();

        if magic_number_buffer != new_magic_number_buffer {
            return Err(Error::LogicError("Flash read-write-read cycle failed!"));
        }
        uprintln!(self.serial, "[POST]: Flash ID verification and RWR cycle passed");
        Ok(())
    }

    pub fn run(self) -> ! {
        loop {}
    }
}
