use super::*;
use crate::devices::update_signal::UpdateSignal;

impl<EXTF: Flash, MCUF: Flash, SRL: Serial, T: time::Now, R: image::Reader, US: UpdateSignal>
    Bootloader<EXTF, MCUF, SRL, T, R, US>
{
    /// Restores the first image available in all banks, attempting to restore
    /// from the golden image as a last resort.
    pub fn restore(&mut self) -> Result<Image<MCUF::Address>, Error> {
        self.restore_internal(false)
            .or_else(|| self.restore_external(false))
            .or_else(|| self.restore_internal(true))
            .or_else(|| self.restore_external(true))
            .ok_or(Error::NoImageToRestoreFrom)
    }

    fn restore_external(&mut self, golden: bool) -> Option<Image<MCUF::Address>> {
        let output = self.boot_bank();
        for input_bank in self.external_banks.iter().filter(|b| b.is_golden == golden) {
            duprintln!(
                self.serial,
                "Attempting to restore from{} bank {:?}.",
                if golden { " golden" } else { "" },
                input_bank.index
            );
            if Self::copy_image(
                &mut self.serial,
                self.external_flash.as_mut().unwrap(),
                &mut self.mcu_flash,
                *input_bank,
                output,
                golden,
            )
            .is_err()
            {
                continue;
            }

            duprintln!(
                self.serial,
                "Restored image from bank {:?} [{}]",
                input_bank.index,
                EXTF::label()
            );
            duprintln!(self.serial, "Verifying the image again in the boot bank...");
            self.boot_metrics.boot_path = BootPath::Restored { bank: input_bank.index };
            return R::image_at(&mut self.mcu_flash, output).ok();
        }
        None
    }

    fn restore_internal(&mut self, golden: bool) -> Option<Image<MCUF::Address>> {
        let output = self.boot_bank();
        for input_bank in
            self.mcu_banks.iter().filter(|b| b.is_golden == golden && b.index != output.index)
        {
            duprintln!(
                self.serial,
                "Attempting to restore from{} bank {:?}.",
                if golden { " golden" } else { "" },
                input_bank.index
            );
            if Self::copy_image_single_flash(
                &mut self.serial,
                &mut self.mcu_flash,
                *input_bank,
                output,
                golden,
            )
            .is_err()
            {
                continue;
            }

            duprintln!(
                self.serial,
                "Restored image from bank {:?} [{}]",
                input_bank.index,
                MCUF::label()
            );
            duprintln!(self.serial, "Verifying the image again in the boot bank...");
            self.boot_metrics.boot_path = BootPath::Restored { bank: input_bank.index };
            return R::image_at(&mut self.mcu_flash, output).ok();
        }
        None
    }
}
