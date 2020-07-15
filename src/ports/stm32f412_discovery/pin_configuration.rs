//! GPIO configuration and alternate functions for the [stm32f412 discovery](../../../../../../documentation/hardware/discovery.pdf).
use crate::drivers::stm32f4::serial::{TxPin, RxPin};
use crate::stm32pac::USART6;
use crate::drivers::stm32f4::qspi::{
    ClkPin as QspiClk,
    Bk1CsPin as QspiChipSelect,
    Bk1Io0Pin as QspiOutput,
    Bk1Io1Pin as QspiInput,
    Bk1Io2Pin as QspiSecondaryOutput,
    Bk1Io3Pin as QspiSecondaryInput,
};

pin_rows!(a, b, c, d, e, f, g, h, i, j, k,);
gpio!(e, [(1, Output<PushPull>),]);
gpio!(b, [(2, AF9 as QspiClk),]);
gpio!(f, [
    (6, AF9 as QspiSecondaryInput),
    (7, AF9 as QspiSecondaryOutput),
    (8, AF10 as QspiOutput),
    (9, AF10 as QspiInput),
]);
gpio!(g, [
    (6, AF10 as QspiChipSelect),
    (14, AF8 as TxPin<USART6>),
    (9, AF8 as RxPin<USART6>),
]);
