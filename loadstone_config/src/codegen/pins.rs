use std::{fs::{File, OpenOptions}, path::Path};
use anyhow::Result;
use quote::{TokenStreamExt, quote};

use crate::{Configuration, port};

pub fn generate<P: AsRef<Path>>(
    autogenerated_folder_path: P,
    configuration: &Configuration,
) -> Result<()> {
    let filename = autogenerated_folder_path.as_ref().join("pin_configuration.rs");
    let mut file = OpenOptions::new().write(true).create(true).truncate(true).open(&filename)?;

    match configuration.port.subfamily() {
        port::Subfamily::Stm32f4 => generate_stm32f4(configuration, &mut file),
        port::Subfamily::Efm32Gg11 => generate_efm32gg(configuration, &mut file),
    }
}

fn generate_efm32gg(configuration: &Configuration, file: &mut File) -> Result<()> {
    todo!()
}

fn generate_stm32f4(configuration: &Configuration, file: &mut File) -> Result<()> {
    let mut code = quote! {
        use blue_hal::{enable_gpio, gpio, gpio_inner, alternate_functions, enable_qspi, enable_spi, enable_serial, pin_rows};
        use blue_hal::paste;
        use blue_hal::drivers::stm32f4::gpio::*;
    };

    if configuration.feature_configuration.serial.enabled() {
        code.append_all(quote! {
            use blue_hal::drivers::stm32f4::serial::{TxPin, RxPin};
            use blue_hal::stm32pac::USART6; // FIXME put it in the configuration file.
        });
    }

    if configuration.memory_configuration.external_flash.is_some() {
        code.append_all(quote! {
            use blue_hal::drivers::stm32f4::qspi::{
                ClkPin as QspiClk,
                Bk1CsPin as QspiChipSelect,
                Bk1Io0Pin as QspiOutput,
                Bk1Io1Pin as QspiInput,
                Bk1Io2Pin as QspiSecondaryOutput,
                Bk1Io3Pin as QspiSecondaryInput,
            };
        });
    }

    code.append_all(
        quote!{} // Pins go here
    );
    todo!()
}
