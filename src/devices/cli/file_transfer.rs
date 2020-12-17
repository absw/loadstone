use crate::{
    hal::serial::{TimeoutRead, Write},
    utilities::xmodem,
};

pub const BLOCK_SIZE: usize = xmodem::PAYLOAD_SIZE;
pub type FileBlock = [u8; BLOCK_SIZE];

const MAX_RETRIES: u32 = 10;

pub trait FileTransfer: TimeoutRead + Write {
    fn blocks(&mut self) -> BlockIterator<Self> {
        BlockIterator { serial: self, received_block: false, finished: false, block_number: 0 }
    }
}

impl<T: TimeoutRead + Write> FileTransfer for T {}

pub struct BlockIterator<'a, S: TimeoutRead + Write + ?Sized> {
    serial: &'a mut S,
    received_block: bool,
    finished: bool,
    block_number: u8,
}

impl<'a, S: TimeoutRead + Write + ?Sized> Iterator for BlockIterator<'a, S> {
    type Item = FileBlock;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        let mut retries = 0;
        let mut buffer = [0u8; xmodem::MAX_PACKET_SIZE];

        'block_loop: while retries < MAX_RETRIES {
            let mut buffer_index = 0usize;

            let message = if self.received_block { xmodem::ACK } else { xmodem::NAK };
            if self.serial.write_char(message as char).is_err() {
                retries += 1;
                continue 'block_loop;
            }
            self.received_block = false;

            loop {
                buffer[buffer_index] = match self.serial.read(xmodem::DEFAULT_TIMEOUT) {
                    Ok(byte) => byte,
                    Err(_) => {
                        retries += 1;
                        continue 'block_loop;
                    }
                };

                if buffer_index == 0 || buffer_index == (xmodem::MAX_PACKET_SIZE - 1) {
                    if let Some(block) = self.process_message(&buffer) {
                        self.received_block = true;
                        return Some(block);
                    }

                    if self.finished {
                        return None;
                    }
                }
                buffer_index += 1;
                if buffer_index == xmodem::MAX_PACKET_SIZE {
                    continue 'block_loop;
                }
            }
        }

        // Fully timed out
        self.finished = true;
        None
    }
}

impl<'a, S: TimeoutRead + Write + ?Sized> BlockIterator<'a, S> {
    fn process_message(&mut self, buffer: &[u8]) -> Option<FileBlock> {
        match xmodem::parse_message(&buffer) {
            Ok((_, xmodem::Message::EndOfTransmission)) => {
                self.end_transmission();
                None
            }
            Ok((_, xmodem::Message::Chunk(chunk))) => {
                if let Some(block) = self.process_chunk(chunk) {
                    self.block_number = self.block_number.wrapping_add(1);
                    Some(block)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn process_chunk(&self, chunk: xmodem::Chunk) -> Option<FileBlock> {
        let next_block = self.block_number.wrapping_add(1);
        (chunk.block_number == next_block).then_some(chunk.payload)
    }

    fn end_transmission(&mut self) {
        self.finished = true;
        if self.serial.write_char(xmodem::ACK as char).is_err() {
            return;
        }
        if let Ok(xmodem::ETB) = self.serial.read(xmodem::DEFAULT_TIMEOUT) {
            // We don't care about this being received, as there's no
            // recovering from a failure here.
            let _ = self.serial.write_char(xmodem::ACK as char);
        }
    }
}
