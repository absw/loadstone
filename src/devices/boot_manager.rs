//! Fully CLI interactive boot manager for the demo application.
//!
//! Loadstone proper has few ways to interact with the external world.
//! It can verify integrity of images through their signatures, boot
//! them based on certain criteria, and recover through a simple serial
//! interface when it fails to do so. This means that in order to
//! communicate with loadstone in more depth, a separate application is
//! needed.
//!
//! The boot manager is the central module of such an application. It
//! exposes a CLI that allows flashing firmware images, erasing them,
//! modifying them in certain ways for testing purposes, and interpreting
//! boot metrics left by Loadstone for the application to consume. Any
//! product that needs to interact with Loadstone can use this module as
//! a starting point.

use super::{
    boot_metrics::{boot_metrics, BootMetrics},
    cli::Cli,
    image,
    traits::{Flash, Serial},
};
use crate::error::Error;
use blue_hal::{hal::flash, stm32pac::SCB};

/// Generic boot manager, composed of a CLI interface to serial and flash
/// functionality. Its behaviour is fully generic, and the
/// [ports module](`crate::ports`) provides constructors for specific chips.
pub struct BootManager<EXTF: Flash, SRL: Serial> {
    pub(crate) external_banks: &'static [image::Bank<<EXTF as flash::ReadWrite>::Address>],
    pub(crate) external_flash: EXTF,
    pub(crate) cli: Option<Cli<SRL>>,
    pub(crate) boot_metrics: Option<BootMetrics>,
}

impl<EXTF: Flash, SRL: Serial> BootManager<EXTF, SRL> {
    /// Provides an iterator over all external flash banks.
    pub fn external_banks(&self) -> impl Iterator<Item = image::Bank<EXTF::Address>> {
        self.external_banks.iter().cloned()
    }

    /// Writes a firmware image to an external flash bank. Takes an iterator over byte
    /// blocks, to easily interface with serial or network protocols like XMODEM or TCP/IP
    /// where information is received in chunks.
    pub fn store_image<I: Iterator<Item = [u8; N]>, const N: usize>(
        &mut self,
        blocks: I,
        bank: image::Bank<EXTF::Address>,
    ) -> Result<(), Error> {
        self.external_flash.write_from_blocks(bank.location, blocks)?;
        Ok(())
    }

    /// Fully erases the external flash bank, ensuring there are no leftover images
    /// and future writes to the external flash are as fast as possible.
    pub fn format_external(&mut self) -> Result<(), Error> {
        nb::block!(self.external_flash.erase())?;
        Ok(())
    }

    /// Triggers a soft system reset.
    pub fn reset(&mut self) -> ! { SCB::sys_reset(); }

    /// Gathers metrics left over in memory by Loadstone, if available, and launches
    /// the command line interface.
    pub fn run(mut self, greeting: &'static str) -> ! {
        self.boot_metrics = {
            let metrics = unsafe { boot_metrics().clone() };
            if metrics.is_valid() {
                Some(metrics)
            } else {
                None
            }
        };
        let mut cli = self.cli.take().unwrap();
        loop {
            cli.run(&mut self, greeting)
        }
    }
}
