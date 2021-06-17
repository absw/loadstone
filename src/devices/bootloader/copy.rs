use super::*;
use crate::devices::update_signal::UpdateSignal;

impl<EXTF: Flash, MCUF: Flash, SRL: Serial, T: time::Now, R: image::Reader, US: UpdateSignal>
    Bootloader<EXTF, MCUF, SRL, T, R, US>
{
    pub fn copy_image_single_flash<F: Flash>(
        serial: &mut Option<SRL>,
        flash: &mut F,
        input_bank: image::Bank<F::Address>,
        output_bank: image::Bank<F::Address>,
        must_be_golden: bool,
    ) -> Result<(), Error> {
        if input_bank.index == output_bank.index {
            return Err(Error::DeviceError("Attempted to copy a bank into itself"));
        }
        let input_image = R::image_at(flash, input_bank)?;
        if must_be_golden && !input_image.is_golden() {
            duprintln!(serial, "Image is not golden.",);
            return Err(Error::DeviceError("Image is not golden"));
        }
        duprintln!(
            serial,
            "Copying bank {:?} image [Address {:?}, size {:?}]\r\n* Input: [{}]\r\n* Output: [{}]",
            input_bank.index,
            input_image.location().into(),
            input_image.size(),
            F::label(),
            F::label(),
        );
        let input_image_start_address = input_bank.location;
        let output_image_start_address = output_bank.location;

        // Large transfer buffer ensures that the number of read-write cycles needed
        // to guarantee flash integrity through the process is minimal.
        const TRANSFER_BUFFER_SIZE: usize = KB!(64);
        let mut buffer = [0u8; TRANSFER_BUFFER_SIZE];
        let mut byte_index = 0usize;

        let total_size = input_image.total_size();

        while byte_index < total_size {
            let bytes_to_read = min(TRANSFER_BUFFER_SIZE, total_size.saturating_sub(byte_index));
            block!(
                flash.read(input_image_start_address + byte_index, &mut buffer[0..bytes_to_read])
            )?;
            block!(flash.write(output_image_start_address + byte_index, &buffer[0..bytes_to_read]))?;
            byte_index += bytes_to_read;
        }
        Ok(())
    }

    pub fn copy_image<I: Flash, O: Flash>(
        serial: &mut Option<SRL>,
        input_flash: &mut I,
        output_flash: &mut O,
        input_bank: image::Bank<I::Address>,
        output_bank: image::Bank<O::Address>,
        must_be_golden: bool,
    ) -> Result<(), Error> {
        let input_image = R::image_at(input_flash, input_bank)?;
        if must_be_golden && !input_image.is_golden() {
            duprintln!(serial, "Image is not golden.",);
            return Err(Error::DeviceError("Image is not golden"));
        }
        duprintln!(
            serial,
            "Copying bank {:?} image [Address {:?}, size {:?}]\r\n* Input: [{}]\r\n* Output: [{}]",
            input_bank.index,
            input_image.location().into(),
            input_image.size(),
            I::label(),
            O::label(),
        );
        let input_image_start_address = input_bank.location;
        let output_image_start_address = output_bank.location;

        // Large transfer buffer ensures that the number of read-write cycles needed
        // to guarantee flash integrity through the process is minimal.
        const TRANSFER_BUFFER_SIZE: usize = KB!(64);
        let mut buffer = [0u8; TRANSFER_BUFFER_SIZE];
        let mut byte_index = 0usize;

        let total_size = input_image.total_size();

        while byte_index < total_size {
            let bytes_to_read = min(TRANSFER_BUFFER_SIZE, total_size.saturating_sub(byte_index));
            block!(input_flash
                .read(input_image_start_address + byte_index, &mut buffer[0..bytes_to_read]))?;
            block!(output_flash
                .write(output_image_start_address + byte_index, &buffer[0..bytes_to_read]))?;
            byte_index += bytes_to_read;
        }
        Ok(())
    }
}
