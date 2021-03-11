use super::*;

impl<EXTF: Flash, MCUF: Flash, SRL: Serial, T: time::Now> Bootloader<EXTF, MCUF, SRL, T> {
    /// If the current bootable (MCU flash) image is different from the top
    /// non-golden image, attempts to replace it. On failure, this process
    /// is repeated for all non-golden banks. Returns the current
    /// bootable image after the process, if available.
    pub fn latest_bootable_image(&mut self) -> Option<Image<MCUF::Address>> {
        duprintln!(self.serial, "Checking for image updates...");
        let boot_bank = self.boot_bank();
        let current_image = if let Ok(image) = image::image_at(&mut self.mcu_flash, boot_bank) {
            image
        } else {
            duprintln!(self.serial, "No current image.");
            return None;
        };

        self.try_update_internal(boot_bank, current_image)
            .or_else(|| self.try_update_external(boot_bank, current_image))
            .or(Some(current_image))
    }

    fn try_update_internal(
        &mut self,
        boot_bank: Bank<MCUF::Address>,
        current_image: Image<MCUF::Address>,
    ) -> Option<Image<MCUF::Address>> {
        for bank in self.mcu_banks().filter(|b| !b.is_golden && b.index != boot_bank.index) {
            duprintln!(
                self.serial,
                "[{}] Scanning bank {:?} for a newer image...",
                MCUF::label(),
                bank.index
            );
            match image::image_at(&mut self.mcu_flash, bank) {
                Ok(image) if image.signature() != current_image.signature() => {
                    if let Some(updated_image) = self.replace_image_internal(bank, boot_bank) {
                        self.boot_metrics.boot_path = BootPath::Updated { bank: bank.index };
                        return Some(updated_image);
                    }
                }
                Ok(_image) => return Some(current_image),
                _ => (),
            }
        }
        None
    }

    fn try_update_external(
        &mut self,
        boot_bank: Bank<MCUF::Address>,
        current_image: Image<MCUF::Address>,
    ) -> Option<Image<MCUF::Address>> {
        if self.external_flash.is_some() {
            for bank in self.external_banks().filter(|b| !b.is_golden) {
                duprintln!(
                    self.serial,
                    "[{}] Scanning bank {:?} for a newer image...",
                    EXTF::label(),
                    bank.index
                );
                match image::image_at(self.external_flash.as_mut().unwrap(), bank) {
                    Ok(image) if image.signature() != current_image.signature() => {
                        if let Some(updated_image) = self.replace_image_external(bank, boot_bank) {
                            self.boot_metrics.boot_path = BootPath::Updated { bank: bank.index };
                            return Some(updated_image);
                        }
                    }
                    Ok(_image) => return Some(current_image),
                    _ => (),
                }
            }
        }
        None
    }

    fn replace_image_internal(
        &mut self,
        bank: Bank<MCUF::Address>,
        boot_bank: Bank<MCUF::Address>,
    ) -> Option<Image<MCUF::Address>> {
        duprintln!(self.serial, "Replacing current image with bank {:?}.", bank.index,);
        Self::copy_image_single_flash(
            &mut self.serial,
            &mut self.mcu_flash,
            bank,
            boot_bank,
            false,
        )
        .unwrap();
        duprintln!(self.serial, "Replaced image with bank {:?} [{}]", bank.index, MCUF::label(),);
        image::image_at(&mut self.mcu_flash, boot_bank).ok()
    }

    fn replace_image_external(
        &mut self,
        bank: Bank<EXTF::Address>,
        boot_bank: Bank<MCUF::Address>,
    ) -> Option<Image<MCUF::Address>> {
        duprintln!(self.serial, "Replacing current image with bank {:?}.", bank.index,);
        Self::copy_image(
            &mut self.serial,
            self.external_flash.as_mut().unwrap(),
            &mut self.mcu_flash,
            bank,
            boot_bank,
            false,
        )
        .unwrap();
        duprintln!(self.serial, "Replaced image with bank {:?} [{}]", bank.index, EXTF::label(),);
        image::image_at(&mut self.mcu_flash, boot_bank).ok()
    }
}
