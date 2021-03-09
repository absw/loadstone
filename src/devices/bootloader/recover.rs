use super::*;

impl<EXTF: Flash, MCUF: Flash, SRL: Serial, T: time::Now> Bootloader<EXTF, MCUF, SRL, T> {
    /// Enters recovery mode, which requests a golden image to be transferred via serial through
    /// the XMODEM protocol, then reboot. If Loadstone has no golden image support, recovery
    /// mode will allow flashing the bootable bank directly.
    pub fn recover(&mut self) -> ! {
        duprintln!(self.serial, "-- Loadstone Recovery Mode --");

        let mcu_golden_bank_exists = self.mcu_banks().any(|b| b.is_golden);
        let external_golden_bank_exists = self.external_banks().any(|b| b.is_golden);
        let no_golden_bank_support = !mcu_golden_bank_exists && !external_golden_bank_exists;

        if mcu_golden_bank_exists {
            match self.recover_internal(true) {
                Ok(_) => {
                    duprintln!(self.serial, "Finished flashing golden image.");
                    self.reboot();
                },
                Err(e) => {
                    duprintln!(self.serial, "FATAL: Image did not flash correctly.");
                    e.report(&mut self.serial);
                    self.reboot();
                }
            }
        }

        if self.external_flash.is_some() && external_golden_bank_exists {
            match self.recover_external(true) {
                Ok(_) => {
                    duprintln!(self.serial, "Finished flashing golden image.");
                    self.reboot();
                },
                Err(e) => {
                    duprintln!(self.serial, "FATAL: Image did not flash correctly.");
                    e.report(&mut self.serial);
                    self.reboot();
                }
            }
        }

        if !no_golden_bank_support {
            match self.recover_internal(false) {
                Ok(_) => {
                    duprintln!(self.serial, "Finished flashing image.");
                    self.reboot();
                },
                Err(e) => {
                    duprintln!(self.serial, "FATAL: Image did not flash correctly.");
                    e.report(&mut self.serial);
                    self.reboot();
                }
            }
        }

        if !no_golden_bank_support && external_golden_bank_exists{
            match self.recover_external(false) {
                Ok(_) => {
                    duprintln!(self.serial, "Finished flashing image.");
                    self.reboot();
                },
                Err(e) => {
                    duprintln!(self.serial, "FATAL: Image did not flash correctly.");
                    e.report(&mut self.serial);
                    self.reboot();
                }
            }
        }


        self.reboot();
    }

    fn reboot(&mut self) -> ! {
        duprintln!(self.serial, "Rebooting...");
        SCB::sys_reset();
    }

    fn recover_internal(&mut self, golden: bool) -> Result<(), Error> {
        if let Some(bank) = self.mcu_banks().find(|b| b.is_golden == golden) {
            duprintln!(self.serial, "Please send{} firmware image via XMODEM.", if golden { " golden" } else { "" });
            let blocks = self.serial.blocks(None);
            if self.mcu_flash.write_from_blocks(bank.location, blocks).is_err() {
                duprintln!(
                    self.serial,
                    "FATAL: Failed to flash{} image during recovery mode.",
                    if golden { " golden" } else { "" },
                );
                panic!();
            }
            match image::image_at(&mut self.mcu_flash, bank) {
                Ok(image) if golden && !image.is_golden() => {
                    duprintln!(self.serial, "FATAL: Flashed image is not a golden image.");
                    Err(Error::ImageIsNotGolden)
                }
                Err(e) => {
                    Err(e)
                }
                _ => {
                    Ok(())
                }
            }
        } else {
            Err(Error::NoGoldenBankSupport)
        }
    }

    fn recover_external(&mut self, golden: bool) -> Result<(), Error> {
        if let Some(bank) = self.external_banks().find(|b| b.is_golden == golden) {
            duprintln!(self.serial, "Please send{} firmware image via XMODEM.", if golden { " golden" } else { "" });
            let blocks = self.serial.blocks(None);
            if self.external_flash.as_mut().unwrap().write_from_blocks(bank.location, blocks).is_err() {
                duprintln!(
                    self.serial,
                    "FATAL: Failed to flash{} image during recovery mode.",
                    if golden { " golden" } else { "" },
                );
                panic!();
            }
            match image::image_at(self.external_flash.as_mut().unwrap(), bank) {
                Ok(image) if golden && !image.is_golden() => {
                    duprintln!(self.serial, "FATAL: Flashed image is not a golden image.");
                    Err(Error::ImageIsNotGolden)
                }
                Err(e) => {
                    Err(e)
                }
                _ => {
                    Ok(())
                }
            }
        } else {
            Err(Error::NoGoldenBankSupport)
        }
    }

}
