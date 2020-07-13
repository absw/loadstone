//! Generic Bootloader.
//!
//! This module contains all bootloader functionality, with
//! the exception of how to construct one. Construction is
//! handled by the `port` module as it depends on board
//! specific information.
use core::convert::Into;
use crate::{
    error::Error,
    hal::{serial, flash::{self, Read, Write}, led},
};
use core::marker::PhantomData;
use nb::block;

pub struct Bootloader<E, A, S, L>
where
    E: flash::Write<A> + flash::Read<A>,
    S: serial::Write<u8>,
    L: led::Toggle,
{
    pub(crate) flash: E,
    pub(crate) post_led: L,
    pub(crate) serial: S,
    pub(crate) _marker: PhantomData<A>,
}

impl<E, A, S, L> Bootloader<E, A, S, L>
where
    E: flash::Write<A> + flash::Read<A>,
    A: Copy + Clone,
    S: serial::Write<u8>,
    L: led::Toggle,
{
    pub fn power_on_self_test(&mut self) -> Result<(), Error> {
        let mut magic_number_buffer = [0u8; 1];
        let mut new_magic_number_buffer = [0u8; 1];

        self.post_led.on();
        let (start, _) = E::writable_range();
        block!(self.flash.read(start, &mut magic_number_buffer)).map_err(Into::into)?;
        new_magic_number_buffer[0] = magic_number_buffer[0].wrapping_add(1);
        block!(self.flash.write(start, &mut new_magic_number_buffer)).map_err(Into::into)?;
        block!(self.flash.read(start, &mut magic_number_buffer)).map_err(Into::into)?;
        self.post_led.off();

        if magic_number_buffer != new_magic_number_buffer {
            return Err(Error::LogicError("Flash read-write-read cycle failed!"));
        }
        uprintln!(self.serial, "[POST]: Flash ID verification and RWR cycle passed");
        Ok(())
    }

    pub fn run(self) -> ! { loop {} }
}
