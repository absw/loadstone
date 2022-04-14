use anyhow::Result;
use quote::{format_ident, quote, TokenStreamExt};
use std::{fs::OpenOptions, io::Write, path::Path};

use crate::{codegen::prettify_file, features::Serial, Configuration};

/// Generates the `devices.rs` module, which contains type definitions and
/// initialisation functions for bootloader features such as serial and external
/// flash.
pub fn generate<P: AsRef<Path>>(
    autogenerated_folder_path: P,
    configuration: &Configuration,
) -> Result<()> {
    let filename = autogenerated_folder_path.as_ref().join("devices.rs");
    let mut file = OpenOptions::new().write(true).create(true).truncate(true).open(&filename)?;
    let mut code = quote! {};

    match configuration.port {
        crate::port::Port::Stm32F412 => {
            generate_serial_stm32(configuration, &mut code)?;
            generate_flash_stm32(configuration, &mut code)?;
        }
        crate::port::Port::Wgm160P => {}
        crate::port::Port::Max32631 => {
            generate_serial_max32(configuration, &mut code)?;
            generate_flash_max32(configuration, &mut code)?;
        }
    }

    file.write_all(format!("{}", code).as_bytes())?;
    prettify_file(filename).ok();
    Ok(())
}

fn generate_flash_stm32(
    configuration: &Configuration,
    code: &mut quote::__private::TokenStream,
) -> Result<()> {
    if configuration.memory_configuration.external_flash.is_some() {
        code.append_all(quote!{
            use blue_hal::hal::time;
            use super::pin_configuration::*;
            pub fn construct_flash(qspi_pins: QspiPins, qspi: stm32pac::QUADSPI) -> Option<ExternalFlash> {
                let qspi_config = qspi::Config::<mode::Single>::default().with_flash_size(24).unwrap();
                let qspi = Qspi::from_config(qspi, qspi_pins, qspi_config).unwrap();
                let external_flash = ExternalFlash::with_timeout(qspi, time::Milliseconds(5000)).unwrap();
                Some(external_flash)
            }
        })
    } else {
        code.append_all(quote!{
            use blue_hal::hal::time;
            use super::pin_configuration::*;
            #[allow(unused)]
            pub fn construct_flash(qspi_pins: QspiPins, qspi: stm32pac::QUADSPI) -> Option<ExternalFlash> { None }
        })
    }
    Ok(())
}

fn generate_flash_max32(config: &Configuration, code: &mut quote::__private::TokenStream) -> Result<()> {
    if config.memory_configuration.external_flash.is_some() {
        code.append_all(quote!{
            use crate::ports::autogenerated::pin_configuration::{ExternalFlash, Spi};
            pub fn construct_flash() -> Option<ExternalFlash> {
                let spi = Spi::new();
                let external_flash = ExternalFlash::new(spi);
                Some(external_flash)
            }
        })
    } else {
        code.append_all(quote!{
            use crate::ports::autogenerated::pin_configuration::ExternalFlash;
            pub fn construct_flash() -> Option<ExternalFlash> { None }
        })
    }
    Ok(())
}

fn generate_serial_stm32(
    configuration: &Configuration,
    code: &mut quote::__private::TokenStream,
) -> Result<()> {
    if let Serial::Enabled { tx_pin, .. } = &configuration.feature_configuration.serial {
        let peripheral = format_ident!("{}", tx_pin.peripheral.to_lowercase());
        code.append_all(quote! {
            use super::pin_configuration::{UsartPins, Serial};
            use blue_hal::stm32pac;
            use blue_hal::drivers::stm32f4::rcc::Clocks;
            use blue_hal::drivers::stm32f4::serial::{self, UsartExt};
            #[allow(unused)]
            pub fn construct_serial(
                serial_pins: UsartPins,
                clocks: Clocks,
                usart1: stm32pac::USART1,
                usart2: stm32pac::USART2,
                usart6: stm32pac::USART6
            ) -> Option<Serial> {
                let serial_config = serial::config::Config::default().baudrate(time::Bps(115200));
                Some(#peripheral.constrain(serial_pins, serial_config, clocks).unwrap())
            }
        });
    } else {
        code.append_all(quote! {
            use super::pin_configuration::{UsartPins, Serial};
            use blue_hal::stm32pac;
            use blue_hal::drivers::stm32f4::rcc::Clocks;
            #[allow(unused)]
            pub fn construct_serial(
                _serial_pins: UsartPins,
                _clocks: Clocks,
                _usart1: stm32pac::USART1,
                _usart2: stm32pac::USART2,
                _usart6: stm32pac::USART6
            ) -> Option<Serial> {
                None
            }
        });
    }
    Ok(())
}

fn generate_serial_max32(_config: &Configuration, _code: &mut quote::__private::TokenStream) -> Result<()> {
    Ok(())
}
