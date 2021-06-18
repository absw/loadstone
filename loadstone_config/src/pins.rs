use serde::{Deserialize, Serialize};
use std::{array::IntoIter, borrow::Cow, fmt::Display};

use crate::port::Port;

// Has to be string-defined as it could be potentially
// anything depending on the target (USART, UART...)
pub type Peripheral = Cow<'static, str>;

// Usually single letter, but not necessarily
pub type Bank = Cow<'static, str>;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Pin {
    pub peripheral: Peripheral,
    pub bank: Bank,
    pub index: u32,
    pub af_index: u32,
}

impl Pin {
    const fn new(peripheral: Cow<'static, str>, bank: Bank, index: u32, af_index: u32) -> Self {
        Self { peripheral, bank, index, af_index }
    }
}

impl Display for Pin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "P{}{}", self.bank, self.index)
    }
}

pub fn serial_tx(port: &Port) -> Box<dyn Iterator<Item = Pin>> {
    match port {
        Port::Stm32F412 => Box::new(IntoIter::new([
            Pin::new(Cow::from("USART1"), Cow::from("a"), 9, 7),
            Pin::new(Cow::from("USART1"), Cow::from("b"), 6, 7),
            Pin::new(Cow::from("USART2"), Cow::from("a"), 2, 7),
            Pin::new(Cow::from("USART2"), Cow::from("d"), 5, 7),
            Pin::new(Cow::from("USART1"), Cow::from("a"), 15, 6),
            Pin::new(Cow::from("USART6"), Cow::from("c"), 6, 8),
            Pin::new(Cow::from("USART6"), Cow::from("a"), 11, 8),
            Pin::new(Cow::from("USART6"), Cow::from("g"), 14, 8),
        ])),
        Port::Wgm160P => Box::new(None.into_iter()),
    }
}

pub fn serial_rx(port: &Port) -> Box<dyn Iterator<Item = Pin>> {
    match port {
        Port::Stm32F412 => Box::new(IntoIter::new([
            Pin::new(Cow::from("USART1"), Cow::from("b"), 3, 7),
            Pin::new(Cow::from("USART1"), Cow::from("b"), 7, 7),
            Pin::new(Cow::from("USART1"), Cow::from("a"), 10, 7),
            Pin::new(Cow::from("USART2"), Cow::from("a"), 3, 7),
            Pin::new(Cow::from("USART2"), Cow::from("d"), 6, 7),
            Pin::new(Cow::from("USART6"), Cow::from("c"), 7, 8),
            Pin::new(Cow::from("USART6"), Cow::from("a"), 12, 8),
            Pin::new(Cow::from("USART6"), Cow::from("g"), 9, 8),
        ])),
        Port::Wgm160P => Box::new(None.into_iter()),
    }
}
