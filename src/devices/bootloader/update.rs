use super::*;
use crate::devices::{cli::file_transfer::FileTransfer, update_signal::{ReadUpdateSignal, UpdatePlan}};

enum UpdateResult<MCUF: Flash> {
    AlreadyUpToDate(Image<MCUF::Address>),
    NotUpdated(Image<MCUF::Address>),
    UpdatedTo(Image<MCUF::Address>),
    UpdateError,
}

impl<
        EXTF: Flash,
        MCUF: Flash,
        SRL: Serial,
        T: time::Now,
        R: image::Reader,
        RUS: ReadUpdateSignal,
        WUS: WriteUpdateSignal,
    > Bootloader<EXTF, MCUF, SRL, T, R, RUS, WUS>
{
    /// If the current bootable (MCU flash) image is different from the top
    /// non-golden image, attempts to replace it. On failure, this process
    /// is repeated for all non-golden banks. Returns the current
    /// bootable image after the process, if available.
    pub fn latest_bootable_image(&mut self) -> Option<Image<MCUF::Address>> {
        let boot_bank = self.boot_bank();
        let current_image = if let Ok(image) = R::image_at(&mut self.mcu_flash, boot_bank) {
            image
        } else {
            duprintln!(self.serial, "No current image.");
            return None;
        };


        let bank: Option<u8> = match self
            .update_signal
            .as_ref()
            .map(|(r, _)| r.read_update_plan())
        {
            None => None,
            Some(UpdatePlan::None) => {
                duprintln!(self.serial, "Update signal set to None, refusing to update.");
                return Some(current_image);
            }
            Some(UpdatePlan::Any) => {
                duprintln!(self.serial, "Update signal set to Any, checking for image updates.");
                None
            }
            Some(UpdatePlan::Index(i)) => {
                duprintln!(
                    self.serial,
                    "Update signal set to Index({}), checking for update in \
                    that bank.",
                    i
                );
                Some(i)
            }
            Some(UpdatePlan::Serial) => {
                duprintln!(self.serial, "Update signal set to Serial, attempting one-shot serial update.");
                return self.attempt_serial_update();
            }
        };

        let current_image = match self.update_internal(boot_bank, current_image, bank) {
            UpdateResult::NotUpdated(current_image) => current_image,
            UpdateResult::AlreadyUpToDate(current_image) => return Some(current_image),
            UpdateResult::UpdatedTo(new_image) => return Some(new_image),
            UpdateResult::UpdateError => return None,
        };

        match self.update_external(boot_bank, current_image, bank) {
            UpdateResult::NotUpdated(current_image) => Some(current_image),
            UpdateResult::AlreadyUpToDate(current_image) => Some(current_image),
            UpdateResult::UpdatedTo(new_image) => Some(new_image),
            UpdateResult::UpdateError => None,
        }
    }

    fn attempt_serial_update(&mut self) -> Option<Image<MCUF::Address>> {
        duprintln!(
            self.serial,
            "Please send firmware image via XMODEM.",
        );
        let bank = self.boot_bank();
        let blocks = self.serial.as_mut().unwrap().blocks(None);
        if self.mcu_flash.write_from_blocks(bank.location, blocks).is_err() {
            duprintln!(self.serial, "FATAL: Failed to flash image during serial update.",);
            panic!();
        }
        R::image_at(&mut self.mcu_flash, bank).ok()
    }

    fn update_internal(
        &mut self,
        boot_bank: Bank<MCUF::Address>,
        current_image: Image<MCUF::Address>,
        target_bank: Option<u8>,
    ) -> UpdateResult<MCUF> {
        for bank in self.mcu_banks().filter(|b| b.index != boot_bank.index) {
            if bank.is_golden {
                duprintln!(
                    self.serial,
                    "[{}] Skipping golden bank {:?} (Golden banks can't be updated from)...",
                    MCUF::label(),
                    bank.index
                );
                continue;
            }

            let skip_nontarget_bank = target_bank.map(|t| t != bank.index).unwrap_or(false);
            if skip_nontarget_bank {
                duprintln!(
                    self.serial,
                    "[{}] Skipping bank {:?} (Update signal was set to a bank index)...",
                    MCUF::label(),
                    bank.index
                );
                continue;
            }

            duprintln!(
                self.serial,
                "[{}] Scanning bank {:?} for a newer image...",
                MCUF::label(),
                bank.index
            );
            match R::image_at(&mut self.mcu_flash, bank) {
                Ok(image) if image.identifier() != current_image.identifier() => {
                    if let Some(updated_image) = self.replace_image_internal(bank, boot_bank) {
                        self.boot_metrics.boot_path = BootPath::Updated { bank: bank.index };
                        return UpdateResult::UpdatedTo(updated_image);
                    } else {
                        return UpdateResult::UpdateError;
                    }
                }
                Ok(_image) => return UpdateResult::AlreadyUpToDate(current_image),
                _ => (),
            }
        }
        UpdateResult::NotUpdated(current_image)
    }

    fn update_external(
        &mut self,
        boot_bank: Bank<MCUF::Address>,
        current_image: Image<MCUF::Address>,
        target_bank: Option<u8>,
    ) -> UpdateResult<MCUF> {
        if self.external_flash.is_some() {
            for bank in self.external_banks() {
                if bank.is_golden {
                    duprintln!(
                        self.serial,
                        "[{}] Skipping golden bank {:?} (Golden banks can't be updated from)...",
                        MCUF::label(),
                        bank.index
                    );
                    continue;
                }

                let skip_nontarget_bank = target_bank.map(|t| t != bank.index).unwrap_or(false);
                if skip_nontarget_bank {
                    duprintln!(
                        self.serial,
                        "[{}] Skipping bank {:?} (Update signal was set to a bank index)...",
                        MCUF::label(),
                        bank.index
                    );
                    continue;
                }

                duprintln!(
                    self.serial,
                    "[{}] Scanning bank {:?} for a newer image...",
                    EXTF::label(),
                    bank.index
                );
                match R::image_at(self.external_flash.as_mut().unwrap(), bank) {
                    Ok(image) if image.identifier() != current_image.identifier() => {
                        if let Some(updated_image) = self.replace_image_external(bank, boot_bank) {
                            self.boot_metrics.boot_path = BootPath::Updated { bank: bank.index };
                            return UpdateResult::UpdatedTo(updated_image);
                        } else {
                            return UpdateResult::UpdateError;
                        }
                    }
                    Ok(_image) => return UpdateResult::AlreadyUpToDate(current_image),
                    _ => (),
                }
            }
        }
        UpdateResult::NotUpdated(current_image)
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
        let image = R::image_at(&mut self.mcu_flash, boot_bank)
            .expect("Failed to verify an image after copy!");
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
        let image = R::image_at(&mut self.mcu_flash, boot_bank)
            .expect("Failed to verify an image after copy!");
        Some(image)
    }
}
