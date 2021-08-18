use anyhow::Result;
use itertools::Itertools;
use quote::{TokenStreamExt, format_ident, quote};
use std::{array::IntoIter, fs::File, io::Write, iter::empty};
use syn::{Ident, Index};

use crate::{Configuration, features::Serial, pins::QspiPins};

struct SerialPinTokens {
    bank: char,
    index: Index,
    mode: Ident,
    direction: Ident,
    peripheral: Ident,
}
struct QspiFlashPinTokens {
    bank: char,
    index: Index,
    mode: Ident,
    earmark: Ident,
}

pub fn generate_stm32f4_pins(configuration: &Configuration, file: &mut File) -> Result<()> {
    let mut code = quote! {
        use blue_hal::{enable_gpio, gpio, gpio_inner, alternate_functions, enable_qspi, enable_spi, enable_serial, pin_rows};
        use blue_hal::drivers::stm32f4::gpio::*;
    };

    generate_imports_and_types(configuration, &mut code);
    generate_gpio_macros(configuration, &mut code);
    generate_pin_constructor(configuration, &mut code);

    file.write_all(format!("{}", code).as_bytes())?;
    Ok(())
}

fn generate_pin_constructor(
    configuration: &Configuration,
    code: &mut quote::__private::TokenStream,
) -> () {
    let banks = 'a'..='h';
    let gpio_fields = banks.clone().map(|b| format_ident!("gpio{}", b)).collect_vec();
    let pac_gpio_fields = banks.map(|b| format_ident!("GPIO{}", b.to_uppercase().next().unwrap()));

    let serial_pin_structs: Box<dyn Iterator<Item = Ident>> =
        if let Serial::Enabled { tx_pin, rx_pin, .. } = &configuration.feature_configuration.serial
        {
            Box::new(IntoIter::new([
                format_ident!("gpio{}", tx_pin.bank),
                format_ident!("gpio{}", rx_pin.bank),
            ]))
        } else {
            Box::new(None.into_iter())
        };

    let serial_pin_fields: Box<dyn Iterator<Item = Ident>> =
        if let Serial::Enabled { tx_pin, rx_pin, .. } = &configuration.feature_configuration.serial
        {
            Box::new(IntoIter::new([
                format_ident!("p{}{}", tx_pin.bank, tx_pin.index),
                format_ident!("p{}{}", rx_pin.bank, rx_pin.index),
            ]))
        } else {
            Box::new(None.into_iter())
        };

    let qspi_pin_structs = qspi_flash_pin_tokens(configuration).map(|p| {
        format_ident!("gpio{}", p.bank)
    });

    let qspi_pin_fields = qspi_flash_pin_tokens(configuration).map(|p| {
        format_ident!("p{}{}", p.bank, p.index)
    });

    code.append_all(quote! {
        #[allow(unused)]
        pub fn pins(#(#gpio_fields: stm32pac::#pac_gpio_fields),*, rcc: &mut stm32pac::RCC) -> (UsartPins, QspiPins) {

            #(let #gpio_fields = #gpio_fields.split(rcc);)*
            (
                (#(#serial_pin_structs.#serial_pin_fields),*),
                (#(#qspi_pin_structs.#qspi_pin_fields),*)
            )
        }
    });
}

fn generate_imports_and_types(
    configuration: &Configuration,
    code: &mut quote::__private::TokenStream,
) {
    if let Serial::Enabled { tx_pin, rx_pin, .. } = &configuration.feature_configuration.serial {
        let peripheral = format_ident!("{}", tx_pin.peripheral);
        let tx_af = format_ident!("AF{}", tx_pin.af_index);
        let tx_pin = format_ident!("P{}{}", tx_pin.bank, tx_pin.index);
        let rx_af = format_ident!("AF{}", rx_pin.af_index);
        let rx_pin = format_ident!("P{}{}", rx_pin.bank, rx_pin.index);

        code.append_all(quote! {
            use blue_hal::drivers::stm32f4::serial::{TxPin, RxPin};
            #[allow(unused_imports)]
            use blue_hal::stm32pac::{self, USART1, USART2, USART6};
            pub type UsartPins = (#tx_pin<#tx_af>, #rx_pin<#rx_af>);
            pub type Serial = blue_hal::drivers::stm32f4::serial::Serial<#peripheral, UsartPins>;
        });
    } else {
        code.append_all(quote! {
            use blue_hal::drivers::stm32f4::serial::{TxPin, RxPin};
            #[allow(unused_imports)]
            use blue_hal::stm32pac::{self, USART1, USART2, USART6};
            pub type UsartPins = ();
            pub type Serial = blue_hal::hal::null::NullSerial;
        });
    }
    if let Some(_) = &configuration.memory_configuration.external_flash {
        let qspi_pins = qspi_flash_pin_tokens(configuration).map(|p| {
            format_ident!("P{}{}", p.bank, p.index)
        });

        let qspi_modes = qspi_flash_pin_tokens(configuration).map(|p| {
            p.mode
        });

        code.append_all(quote! {
            use blue_hal::drivers::micron::n25q128a_flash::MicronN25q128a;
            use blue_hal::drivers::stm32f4::systick::SysTick;
            pub type QspiPins = (#(#qspi_pins<#qspi_modes>,)*);
            pub type Qspi = QuadSpi<QspiPins, mode::Single>;
            pub type ExternalFlash = MicronN25q128a<Qspi, SysTick>;
            #[allow(unused_imports)]
            pub use blue_hal::drivers::stm32f4::qspi::{
                self, mode, QuadSpi,
                ClkPin as QspiClk,
                Bk1CsPin as QspiChipSelect,
                Bk1Io0Pin as QspiOutput,
                Bk1Io1Pin as QspiInput,
                Bk1Io2Pin as QspiSecondaryOutput,
                Bk1Io3Pin as QspiSecondaryInput,
            };
            enable_gpio!();
        });
    } else {
        code.append_all(quote! {
            pub type ExternalFlash = blue_hal::hal::null::NullFlash;
            pub type QspiPins = ();
            enable_gpio!();
        });
    }
}

fn generate_gpio_macros(configuration: &Configuration, code: &mut quote::__private::TokenStream) {
    for bank in 'a'..='h' {
        let serial_tokens = serial_tokens(configuration).filter(|t| t.bank == bank).collect_vec();
        let qspi_flash_pin_tokens =
            qspi_flash_pin_tokens(configuration).filter(|t| t.bank == bank).collect_vec();

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
                #((#serial_index, #serial_mode as #serial_direction<#serial_peripheral>),)*
                #((#qspi_flash_index, #qspi_flash_mode as #qspi_flash_earmark),)*
            ]);
        });
    }
}

fn serial_tokens(configuration: &Configuration) -> Box<dyn Iterator<Item = SerialPinTokens>> {
    if let Serial::Enabled { tx_pin, rx_pin, .. } = &configuration.feature_configuration.serial {
        Box::new(IntoIter::new([
            SerialPinTokens {
                bank: tx_pin.bank.chars().nth(0).unwrap(),
                index: (tx_pin.index as usize).into(),
                mode: format_ident!("AF{}", tx_pin.af_index),
                direction: format_ident!("TxPin"),
                peripheral: format_ident!("{}", tx_pin.peripheral),
            },
            SerialPinTokens {
                bank: rx_pin.bank.chars().nth(0).unwrap(),
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

fn qspi_flash_pin_tokens(
    configuration: &Configuration,
) -> Box<dyn Iterator<Item = QspiFlashPinTokens>> {
    if configuration.memory_configuration.external_flash.is_none() {
        return Box::new(empty());
    }

    let pins = configuration.memory_configuration.external_memory_map.pins.clone()
        .unwrap_or_else(|| QspiPins::create(configuration.port));

    Box::new(IntoIter::new([
        QspiFlashPinTokens {
            bank: pins.clk.bank.chars().next().unwrap(),
            index: (pins.clk.index as usize).into(),
            mode: format_ident!("AF{}", pins.clk.af_index),
            earmark: format_ident!("QspiClk"),
        },
        QspiFlashPinTokens {
            bank: pins.bk1_cs.bank.chars().next().unwrap(),
            index: (pins.bk1_cs.index as usize).into(),
            mode: format_ident!("AF{}", pins.bk1_cs.af_index),
            earmark: format_ident!("QspiChipSelect"),
        },
        QspiFlashPinTokens {
            bank: pins.bk1_io0.bank.chars().next().unwrap(),
            index: (pins.bk1_io0.index as usize).into(),
            mode: format_ident!("AF{}", pins.bk1_io0.af_index),
            earmark: format_ident!("QspiOutput"),
        },
        QspiFlashPinTokens {
            bank: pins.bk1_io1.bank.chars().next().unwrap(),
            index: (pins.bk1_io1.index as usize).into(),
            mode: format_ident!("AF{}", pins.bk1_io1.af_index),
            earmark: format_ident!("QspiInput"),
        },
        QspiFlashPinTokens {
            bank: pins.bk1_io2.bank.chars().next().unwrap(),
            index: (pins.bk1_io2.index as usize).into(),
            mode: format_ident!("AF{}", pins.bk1_io2.af_index),
            earmark: format_ident!("QspiSecondaryOutput"),
        },
        QspiFlashPinTokens {
            bank: pins.bk1_io3.bank.chars().next().unwrap(),
            index: (pins.bk1_io3.index as usize).into(),
            mode: format_ident!("AF{}", pins.bk1_io3.af_index),
            earmark: format_ident!("QspiSecondaryInput"),
        },
    ]))
}
