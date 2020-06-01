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

/// Prints to an abstract serial device
#[macro_export]
macro_rules! uprint {
    ($serial:expr, $arg:tt) => {
        $arg.as_bytes().iter().for_each(|&b| nb::block!($serial.write(b)).unwrap());
    };
}

/// Prints to an abstract serial device, with newline
#[macro_export]
macro_rules! uprintln {
    ($serial:expr, $arg:tt) => {
        uprint!($serial, $arg);
        uprint!($serial, "\n");
    };
}

#[cfg(test)]
mod test {

    #[derive(Debug, Default)]
    struct MockUsart {
        pub mock_value_to_read: u8,
        pub write_record: Vec<u8>,
    }

    impl Write<u8> for MockUsart {
        type Error = ();

        fn write(&mut self, word: u8) -> nb::Result<(), Self::Error> {
            self.write_record.push(word);
            Ok(())
        }
    }

    impl Read<u8> for MockUsart {
        type Error = ();

        /// Reads a single word
        fn read(&mut self) -> nb::Result<u8, Self::Error> {
            Ok(self.mock_value_to_read)
        }
    }

    use super::*;

    #[test]
    fn uprint_macro_writes_bytes_with_no_newline() {
        // Given
        let mut mock_usart = MockUsart::default();
        let arbitrary_message = "Hello world!";
        let arbitrary_message_as_bytes: Vec<u8> = arbitrary_message.as_bytes().iter().cloned().collect();

        // When
        uprint!(mock_usart, arbitrary_message);

        // Then
        assert_eq!(arbitrary_message_as_bytes, mock_usart.write_record);
    }

    #[test]
    fn uprintln_macro_writes_bytes_with_newline() {
        // Given
        let mut mock_usart = MockUsart::default();
        let arbitrary_message = "Hello world with newline!";
        let newline = '\n';
        let mut expected_message: Vec<u8> = arbitrary_message.as_bytes().iter().cloned().collect();
        expected_message.push(newline as u8);

        // When
        uprintln!(mock_usart, arbitrary_message);

        // Then
        assert_eq!(expected_message, mock_usart.write_record);
    }
}
