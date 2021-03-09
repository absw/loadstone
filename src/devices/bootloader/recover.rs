use super::*;

impl<EXTF: Flash, MCUF: Flash, SRL: Serial, T: time::Now> Bootloader<EXTF, MCUF, SRL, T> {
    /// Enters recovery mode, which requests a golden image to be transferred via serial through
    /// the XMODEM protocol, then reboot.
    pub fn recover(&mut self) -> ! {
        duprintln!(self.serial, "-- Loadstone Recovery Mode --");
        duprintln!(self.serial, "Please send golden firmware image via XMODEM.");
        let golden_bank = self.external_banks.iter().find(|b| b.is_golden).unwrap();

        if let Some(ref mut external_flash) = self.external_flash {
            let blocks = self.serial.blocks(None);
            if external_flash.write_from_blocks(golden_bank.location, blocks).is_err() {
                duprintln!(
                    self.serial,
                    "FATAL: Failed to flash golden image during recovery mode."
                );
            }

            match image::image_at(external_flash, *golden_bank) {
                Ok(image) if !image.is_golden() => {
                    duprintln!(self.serial, "FATAL: Flashed image is not a golden image.")
                }
                Err(e) => {
                    duprintln!(self.serial, "FATAL: Image did not flash correctly.");
                    e.report(&mut self.serial);
                }
                _ => duprintln!(self.serial, "Finished flashing golden image."),
            }
        }

        duprintln!(self.serial, "Rebooting...");
        SCB::sys_reset();
    }
}
