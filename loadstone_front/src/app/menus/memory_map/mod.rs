use std::cmp::{self, max};

use crate::app::menus::memory_map::normalize::normalize;

use eframe::egui::{self, Button, Color32, Label, Slider};
use loadstone_config::{KB, memory::{self, Bank, ExternalMemoryMap, FlashChip, InternalMemoryMap}, pins::{PeripheralPin, QspiPins, qspi}, port::Port};

static BOOTLOADER_MAX_LENGTH_KB: u32 = 128;
static GOLDEN_TOOLTIP: &'static str =
    "Mark this bank as golden (used as a fallback in case of corruption)\n \
    Only one non-bootable bank may be golden, and only golden banks can store golden images.";

mod normalize;

/// Renders the menu to configure the entire memory map, consisting of a mandatory internal
/// flash (and its bank distribution, which must contain a bootable bank) and an optional
/// external flash.
pub fn configure_memory_map(
    ui: &mut egui::Ui,
    internal_memory_map: &mut InternalMemoryMap,
    external_memory_map: &mut ExternalMemoryMap,
    external_flash: &mut Option<FlashChip>,
    golden_index: &mut Option<usize>,
    port: &Port,
) {
    let internal_flash = memory::internal_flash(port);

    normalize(
        internal_memory_map,
        external_memory_map,
        &internal_flash,
        external_flash,
        golden_index,
        port,
    );

    ui.group(|ui| {
        ui.horizontal_wrapped(|ui| {
            ui.add(Label::new("Internal flash chip: ").heading());
            ui.add(
                Label::new(internal_flash.name.clone()).heading().text_color(Color32::LIGHT_BLUE),
            );
        });

        ui.separator();
        ui.label("Bootloader:");
        select_bootloader_location(ui, internal_memory_map, &internal_flash);
        select_bootloader_length(ui, internal_memory_map, &internal_flash);
        ui.label("Banks:");
        ui.separator();
        configure_internal_banks(ui, internal_memory_map, &internal_flash, golden_index);
    });

    ui.separator();

    ui.group(|ui| {
        ui.horizontal_wrapped(|ui| {
            ui.set_enabled(memory::external_flash(port).count() > 0);
            ui.add(Label::new("External flash chip:").heading());
            egui::ComboBox::from_id_source("external_flash_chip")
                .selected_text(match external_flash {
                    Some(map) => &map.name,
                    None => "Select external flash (optional)",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(external_flash, None, "None");
                    for chip in memory::external_flash(port) {
                        ui.selectable_value(external_flash, Some(chip.clone()), chip.name);
                    }
                });
        });
        ui.separator();

        if let Some(external_flash) = external_flash {
            ui.label("Banks:");
            ui.separator();
            configure_external_banks(
                ui,
                *port,
                external_memory_map,
                internal_memory_map,
                external_flash,
                golden_index,
            );
        }
    });
}

fn configure_internal_banks(
    ui: &mut egui::Ui,
    internal_memory_map: &mut InternalMemoryMap,
    internal_flash: &memory::FlashChip,
    golden_index: &mut Option<usize>,
) {
    let InternalMemoryMap { banks, bootable_index, .. } = internal_memory_map;
    let mut to_delete: Option<usize> = None;
    for (i, bank) in banks.iter_mut().enumerate() {
        configure_internal_bank(
            ui,
            bank,
            internal_flash,
            bootable_index,
            i,
            golden_index,
            &mut to_delete,
        );
    }

    if let Some(to_delete) = to_delete {
        banks.remove(to_delete);
    }

    let bank_start_address =
        internal_memory_map.banks.last().map(|b| b.end_address()).unwrap_or(max(
            internal_memory_map.bootloader_location
                + internal_memory_map.bootloader_length_kb * KB!(1),
            internal_flash.start + internal_memory_map.bootloader_length_kb * KB!(1),
        ));
    let enough_space = bank_start_address + internal_flash.region_size < internal_flash.end;
    ui.set_enabled(enough_space);
    ui.horizontal_wrapped(|ui| {
        add_internal_bank(
            ui,
            golden_index,
            internal_memory_map,
            bank_start_address,
            internal_flash,
        );
    });
}

fn add_internal_bank(
    ui: &mut egui::Ui,
    golden_index: &mut Option<usize>,
    internal_memory_map: &mut InternalMemoryMap,
    bank_start_address: u32,
    internal_flash: &FlashChip,
) {
    if ui.button("Add bank").clicked() {
        // Bump the golden index if we added a bank under the golden one
        match golden_index {
            Some(index) if *index >= internal_memory_map.banks.len() => *index = *index + 1,
            _ => (),
        };
        internal_memory_map.banks.push(Bank {
            start_address: bank_start_address,
            size_kb: internal_flash.region_size / KB!(1),
        });
    };
    ui.label(format!(
        "({}KB available space)",
        internal_flash.end.saturating_sub(bank_start_address) / KB!(1)
    ));
}

fn configure_internal_bank(
    ui: &mut egui::Ui,
    bank: &mut Bank,
    internal_flash: &FlashChip,
    bootable_index: &mut Option<usize>,
    i: usize,
    golden_index: &mut Option<usize>,
    to_delete: &mut Option<usize>,
) {
    ui.horizontal_wrapped(|ui| {
        ui.add(
            Slider::new(
                &mut bank.size_kb,
                1..=internal_flash.end.saturating_sub(bank.start_address + 1) / KB!(1),
            )
            .clamp_to_range(true)
            .suffix("KB"),
        );
        ui.label(format!("Bank {}", i + 1));
        ui.add(
            Label::new(format!("(0x{:x} - 0x{:x})", bank.start_address, bank.end_address()))
                .text_color(Color32::LIGHT_BLUE),
        );
        ui.radio_value(bootable_index, Some(i), "Bootable");
        ui.scope(|ui| {
            ui.set_enabled(*bootable_index != Some(i));
            if ui.radio(*golden_index == Some(i), "Golden").on_hover_text(GOLDEN_TOOLTIP).clicked()
            {
                *golden_index = match *golden_index {
                    Some(index) if index == i => None,
                    _ => Some(i),
                }
            };
        });
        if ui.add(Button::new("Delete").text_color(Color32::RED).small()).clicked() {
            *to_delete = Some(i);
            if let Some(index) = golden_index {
                if i == *index {
                    *golden_index = None;
                } else if i < *index {
                    *index = *index - 1
                }
            }
        };
    });
}

fn configure_external_banks(
    ui: &mut egui::Ui,
    port: Port,
    external_memory_map: &mut ExternalMemoryMap,
    internal_memory_map: &InternalMemoryMap,
    external_flash: &memory::FlashChip,
    golden_index: &mut Option<usize>,
) {
    let ExternalMemoryMap { pins, banks: external_banks } = external_memory_map;
    let InternalMemoryMap { banks: internal_banks, .. } = internal_memory_map;

    let mut pins_box = pins.is_some();
    ui.horizontal_wrapped(|ui| {
        ui.checkbox(&mut pins_box, "Pins");
        match (pins_box, &pins) {
            (true, None) => {
                *pins = Some(QspiPins::create(port));
            },
            (false, Some(_)) => {
                *pins = None;
            },
            _ => { },
        };
    });

    if let Some(pins) = pins {
        configure_qpsi_pins(ui, port, pins);
    }

    let mut to_delete: Option<usize> = None;
    for (i, bank) in external_banks.iter_mut().enumerate() {
        configure_external_bank(
            i,
            internal_banks,
            ui,
            bank,
            external_flash,
            golden_index,
            &mut to_delete,
        );
    }

    if let Some(to_delete) = to_delete {
        external_banks.remove(to_delete);
    }

    let bank_start_address =
        external_memory_map.banks.last().map(|b| b.end_address()).unwrap_or(external_flash.start);
    let enough_space = bank_start_address + external_flash.region_size < external_flash.end;
    ui.set_enabled(enough_space);
    ui.horizontal_wrapped(|ui| {
        add_external_bank(ui, external_memory_map, bank_start_address, external_flash);
    });
}

fn add_external_bank(
    ui: &mut egui::Ui,
    external_memory_map: &mut ExternalMemoryMap,
    bank_start_address: u32,
    external_flash: &FlashChip,
) {
    if ui.button("Add bank").clicked() {
        external_memory_map.banks.push(Bank {
            start_address: bank_start_address,
            size_kb: external_flash.region_size / KB!(1),
        });
    };
    ui.label(format!(
        "({}KB available space)",
        external_flash.end.saturating_sub(bank_start_address) / KB!(1)
    ));
}

fn configure_external_bank(
    i: usize,
    internal_banks: &Vec<Bank>,
    ui: &mut egui::Ui,
    bank: &mut Bank,
    external_flash: &FlashChip,
    golden_index: &mut Option<usize>,
    to_delete: &mut Option<usize>,
) {
    let global_index = i + internal_banks.len();
    ui.horizontal_wrapped(|ui| {
        ui.add(
            Slider::new(
                &mut bank.size_kb,
                1..=external_flash.end.saturating_sub(bank.start_address + 1) / KB!(1),
            )
            .clamp_to_range(true)
            .suffix("KB"),
        );
        ui.label(format!("Bank {}", global_index + 1));
        ui.add(
            Label::new(format!("(0x{:x} - 0x{:x})", bank.start_address, bank.end_address()))
                .text_color(Color32::LIGHT_BLUE),
        );
        ui.scope(|ui| {
            if ui
                .radio(*golden_index == Some(global_index), "Golden")
                .on_hover_text(GOLDEN_TOOLTIP)
                .clicked()
            {
                *golden_index = match golden_index {
                    Some(index) if *index == global_index => None,
                    _ => Some(global_index),
                }
            };
        });
        if ui.add(Button::new("Delete").text_color(Color32::RED).small()).clicked() {
            *to_delete = Some(i);
            if let Some(index) = golden_index {
                if global_index == *index {
                    *golden_index = None;
                } else if global_index < *index {
                    *index = *index - 1
                }
            }
        };
    });
}

fn select_bootloader_length(
    ui: &mut egui::Ui,
    internal_memory_map: &mut InternalMemoryMap,
    internal_flash: &memory::FlashChip,
) {
    ui.horizontal_wrapped(|ui| {
        ui.add(
            Slider::new(
                &mut internal_memory_map.bootloader_length_kb,
                1..=cmp::min(
                    BOOTLOADER_MAX_LENGTH_KB,
                    (internal_flash.end - internal_memory_map.bootloader_location) / KB!(1),
                ),
            )
            .clamp_to_range(true)
            .suffix("KB"),
        );
        ui.label("Bootloader allocated length");
    });
    if internal_memory_map.bootloader_length_kb < 64 {
        ui.colored_label(
            Color32::YELLOW,
            "You must manually ensure you've allocated enough \
            bootloader space to hold the final compiled binary.",
        );
    }
}

fn select_bootloader_location(
    ui: &mut egui::Ui,
    internal_memory_map: &mut InternalMemoryMap,
    internal_flash: &memory::FlashChip,
) {
    ui.horizontal_wrapped(|ui| {
        ui.add(
            Slider::new(
                &mut internal_memory_map.bootloader_location,
                internal_flash.start
                    ..=(internal_flash.end.saturating_sub(KB!(BOOTLOADER_MAX_LENGTH_KB))),
            )
            .clamp_to_range(true),
        );
        ui.label("Bootloader location");
        ui.add(
            Label::new(format!(
                "(0x{:x} - 0x{:x})",
                internal_memory_map.bootloader_location,
                internal_memory_map.bootloader_location
                    + KB!(internal_memory_map.bootloader_length_kb)
            ))
            .text_color(Color32::LIGHT_BLUE),
        );
    });
}

fn configure_qpsi_pins(ui: &mut egui::Ui, port: Port, pins: &mut QspiPins) {
    let old_pins = [
        pins.clk.clone(),
        pins.bk1_cs.clone(),
        pins.bk1_io0.clone(),
        pins.bk1_io1.clone(),
        pins.bk1_io2.clone(),
        pins.bk1_io3.clone(),
    ];

    let available = qspi(port);
    let mut alternatives = vec![
        available.clk,
        available.bk1_cs,
        available.bk1_io0,
        available.bk1_io1,
        available.bk1_io2,
        available.bk1_io3,
    ];

    let new_pins = [
        &mut pins.clk,
        &mut pins.bk1_cs,
        &mut pins.bk1_io0,
        &mut pins.bk1_io1,
        &mut pins.bk1_io2,
        &mut pins.bk1_io3,
    ];

    let names = [
        "clk",
        "bk1_cs",
        "bk1_io0",
        "bk1_io1",
        "bk1_io2",
        "bk1_io3",
    ];

    for i in 0..6usize {
        let alternatives: Vec<PeripheralPin> = alternatives.remove(0).filter(|p| {
            for o in &old_pins {
                if *o == *p { return false; }
            }
            true
        }).collect();

        egui::ComboBox::from_label(names[i]).selected_text(new_pins[i].to_string()).show_ui(ui, |ui| {
            for alternative in alternatives {
                ui.selectable_value(new_pins[i], alternative.clone(), alternative);
            }
        });
    }
}
