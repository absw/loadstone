use serde::Serialize;

use crate::port::{board, subfamily, Port};

#[macro_export(local_inner_macros)]
macro_rules! KB {
    ($val:expr) => {
        $val * 1024
    };
}

#[derive(Default, Debug, Clone, Serialize)]
pub struct Bank {
    pub start_address: u32,
    pub size_kb: u32,
}

impl Bank {
    pub fn end_address(&self) -> u32 { self.start_address + self.size_kb * 1024 }
}

#[derive(Debug, Clone, Serialize)]
pub struct InternalMemoryMap {
    pub bootloader_location: u32,
    pub bootloader_length_kb: u32,
    pub banks: Vec<Bank>,
    pub bootable_index: Option<usize>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ExternalMemoryMap {
    pub banks: Vec<Bank>,
}

impl Default for InternalMemoryMap {
    fn default() -> Self {
        Self {
            bootloader_location: 0,
            bootloader_length_kb: 64,
            banks: Vec::new(),
            bootable_index: None,
        }
    }
}

#[derive(Default, Clone, Serialize)]
pub struct MemoryConfiguration {
    pub internal_memory_map: InternalMemoryMap,
    pub external_memory_map: ExternalMemoryMap,
    pub external_flash: Option<FlashChip>,
    pub golden_index: Option<usize>,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize)]
pub struct FlashChip {
    pub name: &'static str,
    pub internal: bool,
    pub start: u32,
    pub end: u32,
    /// Size of the smallest erasable region
    pub region_size: u32,
}

pub fn internal_flash(port: &Port) -> Option<FlashChip> {
    if port.board_name() == board::STM32F412 {
        Some(FlashChip {
            name: "STM32F412 MCU Flash",
            internal: true,
            start: 0x0800_0000,
            end: 0x0810_0000,
            region_size: KB!(16),
        })
    } else if port.subfamily_name() == subfamily::EFM32GG11 {
        Some(FlashChip {
            name: "EFM32GG11 MCU Flash",
            internal: true,
            start: 0x0000_0000,
            end: 512 * KB!(4),
            region_size: KB!(4),
        })
    } else {
        None
    }
}

pub fn external_flash(port: &Port) -> impl Iterator<Item = FlashChip> {
    (port.subfamily_name() == subfamily::STM32F4)
        .then_some(FlashChip {
            name: "Micron n25q128a",
            internal: false,
            start: 0x0000_0000,
            end: 0x00FF_FFFF,
            region_size: KB!(4),
        })
        .into_iter()
}
