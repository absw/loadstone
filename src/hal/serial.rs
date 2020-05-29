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

#[macro_export]
macro_rules! uprint {
    ($serial:expr, $arg:tt) => {
        $arg.as_bytes().iter().for_each(|&b| nb::block!($serial.write(b)).unwrap());
    };
}

#[macro_export]
macro_rules! uprintln {
    ($serial:expr, $arg:tt) => {
        uprint!($serial, $arg);
        uprint!($serial, "\n");
    };
}
