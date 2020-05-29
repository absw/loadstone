use nb;

/// UART read half
pub trait Read<Word> {
    type Error;

    /// Reads a single word
    fn read(&mut self) -> nb::Result<Word, Self::Error>;
}

/// UART write half
pub trait Write<Word> {
    type Error;

    /// Writes a single word
    fn write(&mut self, word: Word) -> nb::Result<(), Self::Error>;
}
