use std::{array::IntoIter, borrow::Cow, fmt::Display};
use serde::{Deserialize, Serialize};

use crate::port::Port;

// Has to be string-defined as it could be potentially
// anything depending on the target (USART, UART...)
pub type Peripheral = Cow<'static, str>;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Pin {
    pub peripheral: Peripheral,
    pub bank: char,
    pub index: u32,
}

impl Pin {
    const fn new(peripheral: Cow<'static, str>, bank: char, index: u32) -> Self {
        Self { peripheral, bank, index }
    }
}

impl Display for Pin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "P{}{}", self.bank, self.index)
    }
}

pub fn serial_tx(port: &Port) -> Box<dyn Iterator<Item=Pin>> {
    match port {
        Port::Stm32F412 => Box::new(IntoIter::new([
            Pin::new(Cow::from("USART1"), 'a', 9),
            Pin::new(Cow::from("USART1"), 'b', 6),
            Pin::new(Cow::from("USART2"), 'a', 2),
            Pin::new(Cow::from("USART2"), 'd', 5),
            Pin::new(Cow::from("USART1"), 'a', 15),
            Pin::new(Cow::from("USART6"), 'c', 6),
            Pin::new(Cow::from("USART6"), 'a', 11),
            Pin::new(Cow::from("USART6"), 'g', 14),
        ])),
        Port::Wgm160P => Box::new(None.into_iter()),
    }
}

pub fn serial_rx(port: &Port) -> Box<dyn Iterator<Item=Pin>> {
    match port {
        Port::Stm32F412 => Box::new(IntoIter::new([
            Pin::new(Cow::from("USART1"), 'b', 3),
            Pin::new(Cow::from("USART1"), 'b', 7),
            Pin::new(Cow::from("USART1"), 'a', 10),
            Pin::new(Cow::from("USART2"), 'a', 3),
            Pin::new(Cow::from("USART2"), 'd', 6),
            Pin::new(Cow::from("USART6"), 'c', 7),
            Pin::new(Cow::from("USART6"), 'a', 12),
            Pin::new(Cow::from("USART6"), 'g', 9),
        ])),
        Port::Wgm160P => Box::new(None.into_iter()),
    }
}
