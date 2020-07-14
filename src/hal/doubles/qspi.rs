use crate::hal::qspi::Indirect;
use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct WriteRecord {
    pub instruction: Option<u8>,
    pub address: Option<u32>,
    pub data: Vec<u8>,
    pub dummy_cycles: u8,
}

#[derive(Clone, Debug)]
pub struct ReadRecord {
    pub instruction: Option<u8>,
    pub address: Option<u32>,
    pub length_requested: usize,
    pub dummy_cycles: u8,
}

#[derive(Default)]
pub struct MockQspi {
    pub write_records: Vec<WriteRecord>,
    pub read_records: Vec<ReadRecord>,
    pub to_read: VecDeque<Vec<u8>>,
}

impl MockQspi {
    pub fn clear(&mut self) {
        self.write_records.clear();
        self.read_records.clear();
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
        self.write_records.push(WriteRecord {
            instruction,
            address,
            data: data.unwrap_or_default().iter().cloned().collect(),
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
        self.read_records.push(ReadRecord {
            instruction,
            address,
            length_requested: data.len(),
            dummy_cycles,
        });
        data.iter_mut().zip(self.to_read.pop_front().unwrap_or_default()).for_each(|(o, i)| *o = i);
        Ok(())
    }
}
