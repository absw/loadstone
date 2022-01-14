use serde::{Deserialize, Serialize};

use crate::{pins::QspiPins, port::Port};

/// Helper macro for kilobytes in any type (simply multiplies by 1024).
#[macro_export(local_inner_macros)]
macro_rules! KB {
    ($val:expr) => {
        $val * 1024
    };
}

/// Firmware bank. Meant to store firmware images, but can also store other
/// kinds of data if the application requires it.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Bank {
    /// Bank address in flash memory.
    pub start_address: u32,
    /// Bank size in kilobytes.
    pub size_kb: u32,
}

impl Bank {
    /// Address immediately after the end of this bank.
    pub fn end_address(&self) -> u32 { self.start_address + self.size_kb * 1024 }
}

/// Memory map for an internal (MCU) flash. This must contain the loadstone bootloader itself
/// and a bootable bank.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalMemoryMap {
    pub bootloader_location: u32,
    pub bootloader_length_kb: u32,
    pub banks: Vec<Bank>,
    pub bootable_index: Option<usize>,
}

/// Memory map for an optional external flash chip. This cannot contain a bootable
/// bank, but it may contain a golden bank.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExternalMemoryMap {
    pub pins: Option<QspiPins>,
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

/// Configuration struct that fully defines the memory layout managed by loadstone,
/// including the mandatory internal memory map, an optional external memory map,
/// and golden/bookt bank information.
#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub struct MemoryConfiguration {
    pub internal_memory_map: InternalMemoryMap,
    pub external_memory_map: ExternalMemoryMap,
    pub external_flash: Option<FlashChip>,
    pub golden_index: Option<usize>,
}

impl MemoryConfiguration {
    /// Address from where the application image will boot, coinciding
    /// with the start address of the bootable bank.
    pub fn bootable_address(&self) -> Option<u32> {
        Some(
            self.internal_memory_map
                .banks
                .get(self.internal_memory_map.bootable_index?)?
                .start_address,
        )
    }
}

/// Definition of a flash chip's hardware.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FlashChip {
    /// Tag to identify the hardware.
    pub name: String,
    /// Whether the flash chip is internal (MCU flash) or external (QSPI, etc)
    pub internal: bool,
    /// Start address of the user writable area of flash.
    pub start: u32,
    /// End address of the user writable area of flash.
    pub end: u32,
    /// Size of the smallest erasable region
    pub region_size: u32,
}

/// The MCU flash available for a port. All ports must have exactly one
/// main MCU flash for Loadstone to correctly function.
pub fn internal_flash(port: &Port) -> FlashChip {
    match port {
        Port::Stm32F412 => FlashChip {
            name: "STM32F412 MCU Flash".to_owned(),
            internal: true,
            start: 0x0800_0000,
            end: 0x0810_0000,
            region_size: KB!(16),
        },
        Port::Wgm160P => FlashChip {
            name: "EFM32GG11 MCU Flash".to_owned(),
            internal: true,
            start: 0x0000_0000,
            end: 512 * KB!(4),
            region_size: KB!(4),
        },
        Port::Maxim3263 => FlashChip {
            name: "Maxim3263 MCU Flash".to_owned(),
            internal: true,
            start: 0x0000_0000,
            end: 256 * KB!(8),
            region_size: KB!(8),
        },
    }
}

/// Returns an iterator over all the flash chips compatible with the current
/// port (a driver exists for them).
pub fn external_flash(port: &Port) -> impl Iterator<Item = FlashChip> {
    match port {
        Port::Stm32F412 => Some(FlashChip {
            name: "Micron n25q128a".to_owned(),
            internal: false,
            start: 0x0000_0000,
            end: 0x00FF_FFFF,
            region_size: KB!(4),
        })
        .into_iter(),
        Port::Wgm160P => None.into_iter(),
        Port::Maxim3263 => None.into_iter(),
    }
}
