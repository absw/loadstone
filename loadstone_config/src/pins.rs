use serde::{Deserialize, Serialize};
use std::{array::IntoIter, borrow::Cow, fmt::Display};

use crate::port::Port;

/// Name of a peripheral. Different platforms may assign arbitrary names
/// to these (e.g. USART, UART, QSPI), hence the need to represent it as a string.
pub type Peripheral = Cow<'static, str>;

/// Serial banks such as the "A" in "GPIOA". They are often single letter, but
/// not necessarily; hence the string type.
pub type Bank = Cow<'static, str>;

/// A pin configured to perform a specific peripheral function (as opposed to a raw input/output).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
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

/// Returns an iterator over the possible serial transmission pins for this port.
pub fn serial_tx(port: &Port) -> Box<dyn Iterator<Item = PeripheralPin>> {
    match port {
        Port::Stm32F412 => Box::new(IntoIter::new([
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
    }
}

/// Returns an iterator over the possible serial reception pins for this port.
pub fn serial_rx(port: &Port) -> Box<dyn Iterator<Item = PeripheralPin>> {
    match port {
        Port::Stm32F412 => Box::new(IntoIter::new([
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
    }
}
