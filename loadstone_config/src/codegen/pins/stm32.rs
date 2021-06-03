use std::{array::IntoIter, fs::File, io::Write};
use anyhow::Result;
use itertools::Itertools;
use quote::{TokenStreamExt, format_ident, quote};
use syn::{Ident, Index};

use crate::{Configuration, features::Serial};

struct InputPinTokens {bank: char, index: Index, mode: Ident}
struct SerialPinTokens {bank: char, index: Index, mode: Ident, direction: Ident, peripheral: Ident}
struct QspiFlashPinTokens {bank: char, index: Index, mode: Ident, earmark: Ident}

fn input_tokens(_configuration: &Configuration) -> Box<dyn Iterator<Item=InputPinTokens>> {
    Box::new(IntoIter::new([
        InputPinTokens{bank: 'a', index: 0.into(), mode: format_ident!("Floating")},
        InputPinTokens{bank: 'a', index: 1.into(), mode: format_ident!("Floating")},
    ]))
}

fn serial_tokens(configuration: &Configuration) -> Box<dyn Iterator<Item=SerialPinTokens>> {
    if let Serial::Enabled { tx_pin, rx_pin, .. } = &configuration.feature_configuration.serial {
        Box::new(IntoIter::new([
            SerialPinTokens {
                bank: tx_pin.bank,
                index: (tx_pin.index as usize).into(),
                mode: format_ident!("AF{}", tx_pin.af_index),
                direction: format_ident!("TxPin"),
                peripheral: format_ident!("{}", tx_pin.peripheral),
            },
            SerialPinTokens {
                bank: rx_pin.bank,
                index: (rx_pin.index as usize).into(),
                mode: format_ident!("AF{}", rx_pin.af_index),
                direction: format_ident!("RxPin"),
                peripheral: format_ident!("{}", rx_pin.peripheral),
            },
        ]))
    } else {
        Box::new(None.into_iter())
    }
}

fn qspi_flash_pin_tokens(configuration: &Configuration) -> Box<dyn Iterator<Item=QspiFlashPinTokens>> {
    // TODO parse these from config file. They're currently hardcoded here
    if let Some(_) = &configuration.memory_configuration.external_flash {
        Box::new(IntoIter::new([
            QspiFlashPinTokens {
                bank: 'b',
                index: 2.into(),
                mode: format_ident!("AF9"),
                earmark: format_ident!("QspiClk"),
            },
            QspiFlashPinTokens {
                bank: 'f',
                index: 6.into(),
                mode: format_ident!("AF9"),
                earmark: format_ident!("QspiSecondaryInput"),
            },
            QspiFlashPinTokens {
                bank: 'f',
                index: 7.into(),
                mode: format_ident!("AF9"),
                earmark: format_ident!("QspiSecondaryOutput"),
            },
            QspiFlashPinTokens {
                bank: 'f',
                index: 8.into(),
                mode: format_ident!("AF10"),
                earmark: format_ident!("QspiOutput"),
            },
            QspiFlashPinTokens {
                bank: 'f',
                index: 9.into(),
                mode: format_ident!("AF10"),
                earmark: format_ident!("QspiInput"),
            },
            QspiFlashPinTokens {
                bank: 'g',
                index: 6.into(),
                mode: format_ident!("AF10"),
                earmark: format_ident!("QspiChipSelect"),
            },
        ]))
    } else {
        Box::new(None.into_iter())
    }
}

pub fn generate_stm32f4_pins(configuration: &Configuration, file: &mut File) -> Result<()> {
    let mut code = quote! {
        use blue_hal::{enable_gpio, gpio, gpio_inner, alternate_functions, enable_qspi, enable_spi, enable_serial, pin_rows};
        use blue_hal::paste;
        use blue_hal::drivers::stm32f4::gpio::*;
    };

    if let Serial::Enabled { tx_pin, rx_pin, .. } = &configuration.feature_configuration.serial {
        let peripheral = format_ident!("{}", tx_pin.peripheral);
        let tx_af = format_ident!("AF{}", tx_pin.af_index);
        let tx_pin = format_ident!("P{}{}", tx_pin.bank, tx_pin.index);
        let rx_af = format_ident!("AF{}", rx_pin.af_index);
        let rx_pin = format_ident!("P{}{}", rx_pin.bank, rx_pin.index);

        code.append_all(quote! {
            use blue_hal::drivers::stm32f4::serial::{TxPin, RxPin};
            use blue_hal::stm32pac::#peripheral;
            pub type UsartPins = (#tx_pin<#tx_af>, #rx_pin<#rx_af>);
            pub type Serial = blue_hal::drivers::stm32f4::serial::Serial<#peripheral, UsartPins>;
        });
    }

    if let Some(_) = &configuration.memory_configuration.external_flash {
        code.append_all(quote! {
            use blue_hal::drivers::stm32f4::qspi::{
                ClkPin as QspiClk,
                Bk1CsPin as QspiChipSelect,
                Bk1Io0Pin as QspiOutput,
                Bk1Io1Pin as QspiInput,
                Bk1Io2Pin as QspiSecondaryOutput,
                Bk1Io3Pin as QspiSecondaryInput,
            };

            enable_gpio!();
        });
    }

    let mut gpio_banks = input_tokens(configuration).map(|t| t.bank)
        .chain(serial_tokens(configuration).map(|t| t.bank))
        .chain(qspi_flash_pin_tokens(configuration).map(|t| t.bank))
        .collect_vec();
    gpio_banks.sort();
    gpio_banks.dedup();

    for bank in gpio_banks {
        let input_tokens = input_tokens(configuration).filter(|t| t.bank == bank).collect_vec();
        let serial_tokens = serial_tokens(configuration).filter(|t| t.bank == bank).collect_vec();
        let qspi_flash_pin_tokens = qspi_flash_pin_tokens(configuration).filter(|t| t.bank == bank).collect_vec();

        let input_index = input_tokens.iter().map(|t| &t.index);
        let input_mode = input_tokens.iter().map(|t| &t.mode);

        let serial_index = serial_tokens.iter().map(|t| &t.index);
        let serial_mode = serial_tokens.iter().map(|t| &t.mode);
        let serial_direction = serial_tokens.iter().map(|t| &t.direction);
        let serial_peripheral = serial_tokens.iter().map(|t| &t.peripheral);

        let qspi_flash_index = qspi_flash_pin_tokens.iter().map(|t| &t.index);
        let qspi_flash_mode = qspi_flash_pin_tokens.iter().map(|t| &t.mode);
        let qspi_flash_earmark = qspi_flash_pin_tokens.iter().map(|t| &t.earmark);

        let bank = format_ident!("{}", bank);

        code.append_all(quote! {
            gpio!(#bank, [
                #((#input_index, Input<#input_mode>),)*
                #((#serial_index, #serial_mode as #serial_direction<#serial_peripheral>),)*
                #((#qspi_flash_index, #qspi_flash_mode as #qspi_flash_earmark),)*
            ]);
        });
    }
    file.write_all(format!("{}", code).as_bytes())?;
    Ok(())
}
