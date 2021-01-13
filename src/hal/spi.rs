//! Traits for Serial Peripheral Interface implementation.

// Allows the transmission and reception of a word in full duplex.
pub trait FullDuplex<WORD> {
    type Error;

    fn transmit(&mut self, word: Option<WORD>) -> nb::Result<(), Self::Error>;
    // Must be called after transmit (full duplex operation)
    fn receive(&mut self) -> nb::Result<WORD, Self::Error>;
}
