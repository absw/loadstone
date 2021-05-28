use eframe::egui::{self, CollapsingHeader};
use loadstone_config::{features::BootMetrics, port::{Port, PortLevel}};

pub mod serial;
pub mod memory_map;
pub mod security;
pub mod generate;

pub fn select_port(ui: &mut egui::Ui, port: &mut Port, port_tree: &Vec<PortLevel>) {
    // Displays the most expressive target as label (board before subfamily before family)
    let target_label = format!(
        "Target{}",
        port.board
            .as_ref()
            .or(port.subfamily.as_ref())
            .or(port.family.as_ref())
            .map_or("".to_owned(), |t| format!(" ({})", t.name()))
    );
    CollapsingHeader::new(target_label).default_open(true).show(ui, |ui| {
        select_port_level(ui, &mut port.family, "Select an MCU family", port_tree);
        match &port.family {
            Some(family) if !family.children().is_empty() => {
                select_port_level(ui, &mut port.subfamily, "Select a subfamily", family.children())
            }
            _ => (),
        }
        match &port.subfamily {
            Some(subfamily) if !subfamily.children().is_empty() => {
                select_port_level(ui, &mut port.board, "Select a board", subfamily.children())
            }
            _ => (),
        }
    });
}

pub fn select_port_level(
    ui: &mut egui::Ui,
    selected_port_level: &mut Option<PortLevel>,
    text: &str,
    port_levels: &Vec<PortLevel>,
) {
    ui.horizontal_wrapped(|ui| {
        egui::ComboBox::from_label(text)
            .selected_text(match selected_port_level {
                Some(port_level) => port_level.name(),
                None => "None".into(),
            })
            .show_ui(ui, |ui| {
                for port_level in port_levels.iter() {
                    ui.selectable_value(
                        selected_port_level,
                        Some(port_level.clone()),
                        port_level.name(),
                    );
                }
            });
    });
}

pub fn configure_boot_metrics(ui: &mut egui::Ui, boot_metrics: &mut BootMetrics, port: &Port) {
    let mut metrics_box = matches!(boot_metrics, BootMetrics::Enabled {..});
    ui.horizontal_wrapped(|ui| {
        ui.checkbox(&mut metrics_box, "Boot Metrics");
        match (metrics_box, &boot_metrics) {
            (true, BootMetrics::Disabled) => { *boot_metrics = BootMetrics::Enabled { timing: false }},
            (false, BootMetrics::Enabled{..}) => { *boot_metrics = BootMetrics::Disabled},
            _ => {},
        }
        ui.label("Relay information about the boot process through RAM memory.");
    });
    ui.horizontal_wrapped(|ui| {
        let mut dummy = false;
        let timing_box = if let BootMetrics::Enabled { timing } = boot_metrics { timing } else { &mut dummy };
        ui.separator();
        ui.set_enabled(BootMetrics::timing_supported(port) && metrics_box);
        ui.checkbox(timing_box, "Timing Metrics");
        ui.label("Include boot timing as part of the boot metrics.");
    });
}
