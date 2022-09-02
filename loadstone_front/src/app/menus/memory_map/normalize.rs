use loadstone_config::{
    memory::{self, ExternalMemoryMap, FlashChip, InternalMemoryMap},
    port::Port,
    KB,
};

/// Ensures internal consistency of the memory maps is maintained. Rules like banks
/// staying contiguous, single boot banks, etc are enforced here. This is called
/// after the GUI drives any modification of the memory map, as the invariants can't
/// be easily upheld by the types alone, and having this additional step makes writing
/// the GUI code a lot simpler.
pub fn normalize(
    internal_memory_map: &mut InternalMemoryMap,
    external_memory_map: &mut ExternalMemoryMap,
    internal_flash: &memory::FlashChip,
    external_flash: &mut Option<memory::FlashChip>,
    golden_index: &mut Option<usize>,
    port: &Port,
) {
    enforce_bootable_bank_not_golden(golden_index, internal_memory_map);
    enforce_internal_banks_follow_bootloader(internal_memory_map, internal_flash);
    enforce_internal_banks_are_contiguous(internal_memory_map);
    enforce_internal_bank_ranges_are_maintained(internal_memory_map, internal_flash);

    if let Some(chip) = external_flash {
        if memory::external_flash(port).any(|c| c.name == chip.name) {
            enforce_external_banks_are_contiguous(external_memory_map, chip);
        } else {
            *external_flash = None;
        }
    } else {
        external_memory_map.banks.clear();
    }
}

fn enforce_external_banks_are_contiguous(
    external_memory_map: &mut ExternalMemoryMap,
    chip: &mut FlashChip,
) {
    if external_memory_map.banks.len() > 0 {
        external_memory_map.banks[0].start_address = chip.start;
    }
    if external_memory_map.banks.len() > 1 {
        for i in 0..external_memory_map.banks.len().saturating_sub(1) {
            let pair = &mut external_memory_map.banks[i..=(i + 1)];
            pair[1].start_address = pair[0].end_address();
        }
    }
    external_memory_map
        .banks
        .retain(|b| b.end_address() < chip.end);
}

fn enforce_internal_bank_ranges_are_maintained(
    internal_memory_map: &mut InternalMemoryMap,
    internal_flash: &FlashChip,
) {
    internal_memory_map
        .banks
        .retain(|b| b.end_address() < internal_flash.end);
    if let Some(index) = internal_memory_map.bootable_index {
        if index >= internal_memory_map.banks.len() {
            internal_memory_map.bootable_index = None;
        }
    }
}

fn enforce_internal_banks_are_contiguous(internal_memory_map: &mut InternalMemoryMap) {
    if internal_memory_map.banks.len() > 1 {
        for i in 0..internal_memory_map.banks.len().saturating_sub(1) {
            let pair = &mut internal_memory_map.banks[i..=(i + 1)];
            pair[1].start_address = pair[0].end_address();
        }
    }
}

fn enforce_internal_banks_follow_bootloader(
    internal_memory_map: &mut InternalMemoryMap,
    internal_flash: &FlashChip,
) {
    if internal_memory_map.banks.len() > 0 {
        // The start of the first bank must be aligned to the chip's erase granularity
        internal_memory_map.bootloader_location = internal_memory_map
            .bootloader_location
            .clamp(internal_flash.start, internal_flash.end);

        let bootloader_end = internal_memory_map.bootloader_location
            + KB!(1) * internal_memory_map.bootloader_length_kb;

        let bootloader_end_offset_from_start_of_flash =
            bootloader_end.saturating_sub(internal_flash.start);
        let aligned_offset =
            match bootloader_end_offset_from_start_of_flash % internal_flash.region_size {
                0 => bootloader_end_offset_from_start_of_flash,
                modulo => {
                    bootloader_end_offset_from_start_of_flash
                        + (internal_flash.region_size.saturating_sub(modulo))
                }
            };
        assert!(aligned_offset % internal_flash.region_size == 0);
        let start_of_banks = internal_flash.start + aligned_offset;
        internal_memory_map.banks[0].start_address = start_of_banks;
    }
}

fn enforce_bootable_bank_not_golden(
    golden_index: &mut Option<usize>,
    internal_memory_map: &mut InternalMemoryMap,
) {
    if *golden_index == internal_memory_map.bootable_index {
        *golden_index = None;
    }
}
