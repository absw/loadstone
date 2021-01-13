use crate::hal::spi::FullDuplex;
use std::collections::VecDeque;

pub struct MockSpi<WORD> {
    /// Mock values to be received
    pub to_receive: VecDeque<WORD>,
    /// Mock values sent
    pub sent: VecDeque<WORD>,
    awaiting_receive: bool,
}

impl<WORD> MockSpi<WORD> {
    pub fn new() -> Self {
        Self { to_receive: VecDeque::new(), sent: VecDeque::new(), awaiting_receive: false }
    }
}

impl<WORD: Default> FullDuplex<WORD> for MockSpi<WORD> {
    type Error = ();
    fn transmit(&mut self, word: Option<WORD>) -> nb::Result<(), Self::Error> {
        if self.awaiting_receive {
            Err(nb::Error::Other(()))
        } else {
            self.awaiting_receive = true;
            if let Some(word) = word {
                self.sent.push_back(word)
            }
            Ok(())
        }
    }

    fn receive(&mut self) -> nb::Result<WORD, Self::Error> {
        if !self.awaiting_receive {
            Err(nb::Error::Other(()))
        } else {
            self.awaiting_receive = false;
            Ok(self.to_receive.pop_front().unwrap_or_default())
        }
    }
}
