use super::*;

enum UpdateResult<MCUF: Flash> {
    NotUpdated(Image<MCUF::Address>),
    UpdatedTo(Image<MCUF::Address>),
    UpdateError,
}

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

        let current_image = match self.update_internal(boot_bank, current_image) {
            UpdateResult::NotUpdated(current_image) => current_image,
            UpdateResult::UpdatedTo(new_image) => return Some(new_image),
            UpdateResult::UpdateError => return None,
        };

        match self.update_external(boot_bank, current_image) {
            UpdateResult::NotUpdated(current_image) => Some(current_image),
            UpdateResult::UpdatedTo(new_image) => Some(new_image),
            UpdateResult::UpdateError => None,
        }
    }

    fn update_internal(
        &mut self,
        boot_bank: Bank<MCUF::Address>,
        current_image: Image<MCUF::Address>,
    ) -> UpdateResult<MCUF> {
        for bank in self.mcu_banks().filter(|b| !b.is_golden && b.index != boot_bank.index) {
            duprintln!(
                self.serial,
                "[{}] Scanning bank {:?} for a newer image...",
                MCUF::label(),
                bank.index
            );
            match image::image_at(&mut self.mcu_flash, bank) {
                Ok(image) if image.identifier() != current_image.identifier() => {
                    if let Some(updated_image) = self.replace_image_internal(bank, boot_bank) {
                        self.boot_metrics.boot_path = BootPath::Updated { bank: bank.index };
                        return UpdateResult::UpdatedTo(updated_image);
                    } else {
                        return UpdateResult::UpdateError;
                    }
                }
                Ok(_image) => return UpdateResult::NotUpdated(current_image),
                _ => (),
            }
        }
        return UpdateResult::NotUpdated(current_image);
    }

    fn update_external(
        &mut self,
        boot_bank: Bank<MCUF::Address>,
        current_image: Image<MCUF::Address>,
    ) -> UpdateResult<MCUF> {
        if self.external_flash.is_some() {
            for bank in self.external_banks().filter(|b| !b.is_golden) {
                duprintln!(
                    self.serial,
                    "[{}] Scanning bank {:?} for a newer image...",
                    EXTF::label(),
                    bank.index
                );
                match image::image_at(self.external_flash.as_mut().unwrap(), bank) {
                    Ok(image) if image.identifier() != current_image.identifier() => {
                        if let Some(updated_image) = self.replace_image_external(bank, boot_bank) {
                            self.boot_metrics.boot_path = BootPath::Updated { bank: bank.index };
                            return UpdateResult::UpdatedTo(updated_image);
                        } else {
                            return UpdateResult::UpdateError;
                        }
                    }
                    Ok(_image) => return UpdateResult::NotUpdated(current_image),
                    _ => (),
                }
            }
        }
        return UpdateResult::NotUpdated(current_image);
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
        .expect("Failed to copy a valid image!");
        duprintln!(self.serial, "Replaced image with bank {:?} [{}]", bank.index, MCUF::label(),);
        let image = image::image_at(&mut self.mcu_flash, boot_bank).expect("Failed to verify an image after copy!");
        Some(image)
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
        .expect("Failed to copy a valid image!");
        duprintln!(self.serial, "Replaced image with bank {:?} [{}]", bank.index, MCUF::label(),);
        let image = image::image_at(&mut self.mcu_flash, boot_bank).expect("Failed to verify an image after copy!");
        Some(image)
    }
}
