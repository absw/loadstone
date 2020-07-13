//! Generic Bootloader.
//!
//! This module contains all bootloader functionality, with
//! the exception of how to construct one. Construction is
//! handled by the `port` module as it depends on board
//! specific information.

use crate::hal::flash;

struct Flash<I, E>
where
    I: flash::Write<u8>,
    E: flash::Write<u8> + flash::Read<u8>,
{
    internal: I,
    external: E,
}

pub struct Bootloader<I, E>
where
    I: flash::Write<u8>,
    E: flash::Write<u8> + flash::Read<u8>,
{
    flash: Flash<I, E>,
}

impl<I, E> Bootloader<I, E>
where
    I: flash::Write<u8>,
    E: flash::Write<u8> + flash::Read<u8>,
{
    pub fn run(self) -> ! { loop {} }
}
