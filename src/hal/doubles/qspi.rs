use crate::hal::qspi::Indirect;
use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct CommandRecord {
    pub instruction: Option<u8>,
    pub address: Option<u32>,
    pub data: Option<Vec<u8>>,
    pub length_requested: usize,
    pub dummy_cycles: u8,
}

impl CommandRecord {
    pub fn contains(&self, data: &[u8]) -> bool {
        if let Some(stored) = &self.data {
            data.len() == stored.len() && stored.iter().zip(data.iter()).all(|(a, b)| a == b)
        } else {
            false
        }
    }
}

#[derive(Default)]
pub struct MockQspi {
    pub command_records: Vec<CommandRecord>,
    pub to_read: VecDeque<Vec<u8>>,
}

impl MockQspi {
    pub fn clear(&mut self) {
        self.command_records.clear();
        self.to_read.clear();
    }
}

impl Indirect for MockQspi {
    type Error = ();

    fn write(
        &mut self,
        instruction: Option<u8>,
        address: Option<u32>,
        data: Option<&[u8]>,
        dummy_cycles: u8,
    ) -> nb::Result<(), Self::Error> {
        self.command_records.push(CommandRecord {
            instruction,
            address,
            data: Some(data.unwrap_or_default().to_vec()),
            length_requested: 0,
            dummy_cycles,
        });
        Ok(())
    }

    fn read(
        &mut self,
        instruction: Option<u8>,
        address: Option<u32>,
        data: &mut [u8],
        dummy_cycles: u8,
    ) -> nb::Result<(), Self::Error> {
        self.command_records.push(CommandRecord {
            instruction,
            address,
            data: Some(data.to_vec()),
            length_requested: data.len(),
            dummy_cycles,
        });
        data.iter_mut().zip(self.to_read.pop_front().unwrap_or_default()).for_each(|(o, i)| *o = i);
        Ok(())
    }
}
