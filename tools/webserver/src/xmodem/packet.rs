pub const PAYLOAD_SIZE: usize = 128;
const HEADER_SIZE: usize = 3;
const FOOTER_SIZE: usize = 1;

#[derive(Debug)]
pub enum Packet {
    Data([u8; PAYLOAD_SIZE + HEADER_SIZE + FOOTER_SIZE]),
    Terminal,
}

impl Packet {
    const START_OF_HEADER: u8 = 0x01;
    const END_OF_TRANSMISSION: u8 = 0x04;
    const END_OF_TRANSMISSION_BLOCK: u8 = 0x17;
    const TERMINAL_PACKET : [u8; 1] = [Self::END_OF_TRANSMISSION];

    pub fn new(block_number: u8, payload: &[u8]) -> Self {
        assert!(payload.len() <= PAYLOAD_SIZE);
        let mut data = [0u8; PAYLOAD_SIZE + HEADER_SIZE + FOOTER_SIZE];
        data[0] = Self::START_OF_HEADER;
        data[1] = block_number;
        data[2] = 255u8 - block_number;
        let mut checksum = 0u8;
        for (datum, source) in data.iter_mut()
            .skip(HEADER_SIZE)
            .zip(payload) {
            *datum = *source;
            checksum = checksum.wrapping_add(*datum);
        }
        for padding in data.iter_mut()
            .skip(HEADER_SIZE + payload.len())
            .take(PAYLOAD_SIZE - payload.len()) {
            *padding = Self::END_OF_TRANSMISSION_BLOCK;
            checksum = checksum.wrapping_add(*padding);
        }
        data[HEADER_SIZE + PAYLOAD_SIZE] = checksum;
        Packet::Data(data)
    }

    pub fn data(&self) -> &[u8] {
        match self {
            Packet::Data(d) => d,
            Packet::Terminal => &Self::TERMINAL_PACKET,
        }
    }
}
