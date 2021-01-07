//! Xmodem parser.

use core::convert::TryInto;
use nom::{
    branch::alt,
    bytes::streaming::{tag, take},
    number::streaming::be_u8,
    IResult,
};

use crate::hal::time::Seconds;

pub const PAYLOAD_SIZE: usize = 128;
pub const MAX_PACKET_SIZE: usize = 132;
pub const DEFAULT_TIMEOUT: Seconds = Seconds(3);

pub const ACK: u8 = 0x06;
pub const NAK: u8 = 0x15;
pub const SOH: u8 = 0x01;
pub const EOT: u8 = 0x04;
pub const ETB: u8 = 0x17;
pub const CAN: u8 = 0x18;

#[derive(Debug, Eq, PartialEq)]
pub struct Chunk {
    pub block_number: u8,
    pub payload: [u8; PAYLOAD_SIZE],
}

#[derive(Debug, Eq, PartialEq)]
pub enum Message {
    Chunk(Chunk),
    EndOfTransmission,
    EndOfTransmissionBlock,
    Cancel,
}

pub fn parse_message(input: &[u8]) -> IResult<&[u8], Message> {
    alt((parse_chunk, parse_eot, parse_etb, parse_cancel))(input)
}

fn parse_chunk(input: &[u8]) -> IResult<&[u8], Message> {
    let (input, _) = tag(&[SOH])(input)?;
    let (input, block_number) = be_u8(input)?;
    let (input, _) = tag(&[!block_number])(input)?;
    let (input, payload) = take(PAYLOAD_SIZE)(input)?;
    let checksum: u8 = payload.iter().fold(0u8, |sum, b| sum.wrapping_add(*b));
    let (input, _) = tag(&[checksum])(input)?;
    Ok((input, Message::Chunk(Chunk { block_number, payload: payload.try_into().unwrap() })))
}

fn parse_eot(input: &[u8]) -> IResult<&[u8], Message> {
    Ok((tag(&[EOT])(input)?.0, Message::EndOfTransmission))
}

fn parse_etb(input: &[u8]) -> IResult<&[u8], Message> {
    Ok((tag(&[ETB])(input)?.0, Message::EndOfTransmissionBlock))
}

fn parse_cancel(input: &[u8]) -> IResult<&[u8], Message> {
    Ok((tag(&[CAN])(input)?.0, Message::Cancel))
}

#[cfg(test)]
mod test {
    use super::*;
    use nom::Err::Incomplete;
    const MAX_PACKET_SIZE: usize = 132;

    fn write_test_packet(index: u8, payload_value: u8, buffer: &mut [u8]) {
        let checksum = (0..128).fold(0, |sum: u8, _| sum.wrapping_add(payload_value));
        buffer.iter_mut().enumerate().for_each(|(i, b)| {
            *b = match i {
                0 => SOH,
                1 => index,
                2 => !index,
                3..=130 => payload_value,
                131 => checksum,
                _ => *b,
            }
        });
    }

    #[test]
    fn parsing_single_character_control_messages() {
        let input = [EOT];
        let (input, message) = parse_message(&input).unwrap();
        assert_eq!(Message::EndOfTransmission, message);
        assert_eq!(input.len(), 0);

        let input = [ETB];
        let (input, message) = parse_message(&input).unwrap();
        assert_eq!(Message::EndOfTransmissionBlock, message);
        assert_eq!(input.len(), 0);

        let input = [CAN];
        let (input, message) = parse_message(&input).unwrap();
        assert_eq!(Message::Cancel, message);
        assert_eq!(input.len(), 0);
    }

    #[test]
    fn parsing_complete_input_chunk() {
        let mut input = [0u8; MAX_PACKET_SIZE];
        write_test_packet(7, 42, &mut input);
        let (input, message) = parse_message(&input).unwrap();

        let expected_payload = [42u8; PAYLOAD_SIZE];
        let expected_index = 7u8;

        assert_eq!(
            Message::Chunk(Chunk { payload: expected_payload, block_number: expected_index }),
            message
        );
        assert_eq!(input.len(), 0);
    }

    #[test]
    fn parsing_incomplete_input_chunk() {
        let mut input = [0u8; MAX_PACKET_SIZE / 2];
        write_test_packet(7, 42, &mut input);
        assert!(parse_message(&input).unwrap_err().is_incomplete());
    }

    #[test]
    fn parsing_three_messages_in_a_row() {
        let mut input = [0u8; 2 * MAX_PACKET_SIZE + 1];
        write_test_packet(1, 1, &mut input);
        write_test_packet(2, 2, &mut input[MAX_PACKET_SIZE..]);
        input[2 * MAX_PACKET_SIZE] = EOT;

        let (input, message) = parse_message(&input).unwrap();
        assert_eq!(
            Message::Chunk(Chunk { payload: [1u8; PAYLOAD_SIZE], block_number: 1 }),
            message
        );
        let (input, message) = parse_message(&input).unwrap();
        assert_eq!(
            Message::Chunk(Chunk { payload: [2u8; PAYLOAD_SIZE], block_number: 2 }),
            message
        );
        let (input, message) = parse_message(&input).unwrap();
        assert_eq!(Message::EndOfTransmission, message);
        assert_eq!(input.len(), 0);
    }
}
