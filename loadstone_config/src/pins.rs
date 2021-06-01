use crate::port::Port;

#[derive(Copy, Clone, Debug)]
pub struct Pin {
    pub peripheral: usize,
    pub bank: char,
    pub index: usize,
}

impl Pin {
    const fn new(peripheral: usize, bank: char, index: usize) -> Self {
        Self { peripheral, bank, index }
    }
}

static STM32F4_SERIAL_TX_PINS: &'static [Pin] = &[
    Pin::new(1, 'a', 9),
    Pin::new(1, 'b', 6),
    Pin::new(2, 'a', 2),
    Pin::new(2, 'd', 5),
    Pin::new(1, 'a', 15),
    Pin::new(6, 'c', 7),
    Pin::new(6, 'a', 11),
    Pin::new(6, 'g', 14),
];
static STM32F4_SERIAL_RX_PINS: &'static [Pin] = &[
    Pin::new(1, 'b', 3),
    Pin::new(1, 'b', 7),
    Pin::new(1, 'a', 10),
    Pin::new(2, 'a', 3),
    Pin::new(2, 'd', 6),
    Pin::new(6, 'c', 7),
    Pin::new(6, 'a', 12),
    Pin::new(6, 'g', 9),
];

pub fn serial_tx(port: &Port) -> impl Iterator {
    match port {
        Port::Stm32F412 => STM32F4_SERIAL_TX_PINS.iter().cloned(),
        Port::Wgm160P => [].iter().cloned(),
    }
}

pub fn serial_rx(port: &Port) -> impl Iterator {
    match port {
        Port::Stm32F412 => STM32F4_SERIAL_RX_PINS.iter().cloned(),
        Port::Wgm160P => [].iter().cloned(),
    }
}
