use crate::port::{board_names, Port};

static STM32F412_SERIAL_TX_PINS: &'static [&'static str] =
    &["Pa9", "Pb6", "Pa2", "Pd5", "Pa15", "Pc7", "Pa11", "Pg14"];
static STM32F412_SERIAL_RX_PINS: &'static [&'static str] =
    &["Pb3", "Pb7", "Pa10", "Pa3", "Pd6", "Pc7", "Pa12", "Pg9"];

pub fn serial_tx(port: &Port) -> impl Iterator<Item = &'static str> {
    if port.board_name() == board_names::STM32F412 {
        STM32F412_SERIAL_TX_PINS.iter().cloned()
    } else {
        [].iter().cloned()
    }
}

pub fn serial_rx(port: &Port) -> impl Iterator<Item = &'static str> {
    if port.board_name() == board_names::STM32F412 {
        STM32F412_SERIAL_RX_PINS.iter().cloned()
    } else {
        [].iter().cloned()
    }
}
