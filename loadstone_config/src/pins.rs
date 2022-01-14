use serde::{Deserialize, Serialize};
use std::{borrow::Cow, fmt::Display};

use crate::port::Port;

/// Name of a peripheral. Different platforms may assign arbitrary names
/// to these (e.g. USART, UART, QSPI), hence the need to represent it as a string.
pub type Peripheral = Cow<'static, str>;

/// Serial banks such as the "A" in "GPIOA". They are often single letter, but
/// not necessarily; hence the string type.
pub type Bank = Cow<'static, str>;

/// A pin configured to perform a specific peripheral function (as opposed to a raw input/output).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PeripheralPin {
    /// Associated peripheral for this pin.
    pub peripheral: Peripheral,
    /// Pin bank (the "B" in PB1).
    pub bank: Bank,
    /// Pin index (the "1" in PB1).
    pub index: u32,
    /// Alternate function when the pin is configured to use it.
    pub af_index: u32,
}

impl PeripheralPin {
    const fn new(peripheral: Cow<'static, str>, bank: Bank, index: u32, af_index: u32) -> Self {
        Self { peripheral, bank, index, af_index }
    }
}

impl Display for PeripheralPin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "P{}{}", self.bank, self.index)
    }
}

pub type PinIterator = Box<dyn Iterator<Item = PeripheralPin>>;

/// Returns an iterator over the possible serial transmission pins for this port.
pub fn serial_tx(port: &Port) -> PinIterator {
    match port {
        Port::Stm32F412 => Box::new(IntoIterator::into_iter([
            PeripheralPin::new(Cow::from("USART1"), Cow::from("a"), 9, 7),
            PeripheralPin::new(Cow::from("USART1"), Cow::from("b"), 6, 7),
            PeripheralPin::new(Cow::from("USART2"), Cow::from("a"), 2, 7),
            PeripheralPin::new(Cow::from("USART2"), Cow::from("d"), 5, 7),
            PeripheralPin::new(Cow::from("USART1"), Cow::from("a"), 15, 6),
            PeripheralPin::new(Cow::from("USART6"), Cow::from("c"), 6, 8),
            PeripheralPin::new(Cow::from("USART6"), Cow::from("a"), 11, 8),
            PeripheralPin::new(Cow::from("USART6"), Cow::from("g"), 14, 8),
        ])),
        Port::Wgm160P => Box::new(None.into_iter()),
        Port::Maxim3263 => Box::new(None.into_iter()),
    }
}

/// Returns an iterator over the possible serial reception pins for this port.
pub fn serial_rx(port: &Port) -> PinIterator {
    match port {
        Port::Stm32F412 => Box::new(IntoIterator::into_iter([
            PeripheralPin::new(Cow::from("USART1"), Cow::from("b"), 3, 7),
            PeripheralPin::new(Cow::from("USART1"), Cow::from("b"), 7, 7),
            PeripheralPin::new(Cow::from("USART1"), Cow::from("a"), 10, 7),
            PeripheralPin::new(Cow::from("USART2"), Cow::from("a"), 3, 7),
            PeripheralPin::new(Cow::from("USART2"), Cow::from("d"), 6, 7),
            PeripheralPin::new(Cow::from("USART6"), Cow::from("c"), 7, 8),
            PeripheralPin::new(Cow::from("USART6"), Cow::from("a"), 12, 8),
            PeripheralPin::new(Cow::from("USART6"), Cow::from("g"), 9, 8),
        ])),
        Port::Wgm160P => Box::new(None.into_iter()),
        Port::Maxim3263 => Box::new(None.into_iter()),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QspiPins {
    pub clk: PeripheralPin,
    pub bk1_cs: PeripheralPin,
    pub bk1_io0: PeripheralPin,
    pub bk1_io1: PeripheralPin,
    pub bk1_io2: PeripheralPin,
    pub bk1_io3: PeripheralPin,
}

impl QspiPins {
    pub fn create(port: Port) -> Self {
        assert!(matches!(port, Port::Stm32F412));
        QspiPins {
            clk:     PeripheralPin { peripheral: "QSPI".into(), bank: "b".into(), index: 2, af_index: 9  },
            bk1_cs:  PeripheralPin { peripheral: "QSPI".into(), bank: "g".into(), index: 6, af_index: 10 },
            bk1_io0: PeripheralPin { peripheral: "QSPI".into(), bank: "f".into(), index: 8, af_index: 10 },
            bk1_io1: PeripheralPin { peripheral: "QSPI".into(), bank: "f".into(), index: 9, af_index: 10 },
            bk1_io2: PeripheralPin { peripheral: "QSPI".into(), bank: "f".into(), index: 7, af_index: 9  },
            bk1_io3: PeripheralPin { peripheral: "QSPI".into(), bank: "f".into(), index: 6, af_index: 9  },
        }
    }
}

pub struct QspiPinOptions {
    pub clk: PinIterator,
    pub bk1_cs: PinIterator,
    pub bk1_io0: PinIterator,
    pub bk1_io1: PinIterator,
    pub bk1_io2: PinIterator,
    pub bk1_io3: PinIterator,
}

pub fn qspi(port: Port) -> QspiPinOptions {
    match port {
        Port::Stm32F412 => QspiPinOptions {
            clk: Box::new(IntoIterator::into_iter([
                PeripheralPin::new(Cow::from("QSPI"), Cow::from("b"), 1, 9),
                PeripheralPin::new(Cow::from("QSPI"), Cow::from("b"), 2, 9),
                PeripheralPin::new(Cow::from("QSPI"), Cow::from("d"), 3, 9),
            ])),
            bk1_cs: Box::new(IntoIterator::into_iter([
                PeripheralPin::new(Cow::from("QSPI"), Cow::from("b"), 6, 10),
                PeripheralPin::new(Cow::from("QSPI"), Cow::from("g"), 6, 10),
            ])),
            bk1_io0: Box::new(IntoIterator::into_iter([
                PeripheralPin::new(Cow::from("QSPI"), Cow::from("c"), 9, 9),
                PeripheralPin::new(Cow::from("QSPI"), Cow::from("d"), 11, 9),
                PeripheralPin::new(Cow::from("QSPI"), Cow::from("f"), 8, 10),
            ])),
            bk1_io1: Box::new(IntoIterator::into_iter([
                PeripheralPin::new(Cow::from("QSPI"), Cow::from("c"), 10, 9),
                PeripheralPin::new(Cow::from("QSPI"), Cow::from("d"), 12, 9),
                PeripheralPin::new(Cow::from("QSPI"), Cow::from("f"), 9, 10),
            ])),
            bk1_io2: Box::new(IntoIterator::into_iter([
                PeripheralPin::new(Cow::from("QSPI"), Cow::from("c"), 8, 9),
                PeripheralPin::new(Cow::from("QSPI"), Cow::from("e"), 2, 9),
                PeripheralPin::new(Cow::from("QSPI"), Cow::from("f"), 7, 9),
            ])),
            bk1_io3: Box::new(IntoIterator::into_iter([
                PeripheralPin::new(Cow::from("QSPI"), Cow::from("a"), 1, 10),
                PeripheralPin::new(Cow::from("QSPI"), Cow::from("d"), 13, 10),
                PeripheralPin::new(Cow::from("QSPI"), Cow::from("f"), 6, 9),
            ])),
        },
        Port::Wgm160P => QspiPinOptions {
            clk: Box::new(IntoIterator::into_iter([])),
            bk1_cs: Box::new(IntoIterator::into_iter([])),
            bk1_io0: Box::new(IntoIterator::into_iter([])),
            bk1_io1: Box::new(IntoIterator::into_iter([])),
            bk1_io2: Box::new(IntoIterator::into_iter([])),
            bk1_io3: Box::new(IntoIterator::into_iter([])),
        },
        Port::Maxim3263 => QspiPinOptions {
            clk: Box::new(IntoIterator::into_iter([])),
            bk1_cs: Box::new(IntoIterator::into_iter([])),
            bk1_io0: Box::new(IntoIterator::into_iter([])),
            bk1_io1: Box::new(IntoIterator::into_iter([])),
            bk1_io2: Box::new(IntoIterator::into_iter([])),
            bk1_io3: Box::new(IntoIterator::into_iter([])),
        },
    }
}
