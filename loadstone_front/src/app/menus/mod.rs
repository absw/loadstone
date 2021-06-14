use eframe::egui;
use enum_iterator::IntoEnumIterator;
use loadstone_config::{features::BootMetrics, port::Port};

pub mod serial;
pub mod memory_map;
pub mod security;
pub mod generate;
pub mod update_signal;

pub fn select_port(ui: &mut egui::Ui, port: &mut Port) {
    ui.horizontal_wrapped(|ui| {
        egui::ComboBox::from_label(format!(
            "Family [{}] - Subfamily [{}]",
            port.family(),
            port.subfamily()
        ))
        .selected_text(port.to_string())
        .show_ui(ui, |ui| {
            for port_choice in Port::into_enum_iter() {
                ui.selectable_value(port, port_choice, port_choice.to_string());
            }
        });
    });
}

pub fn configure_boot_metrics(ui: &mut egui::Ui, boot_metrics: &mut BootMetrics, port: &Port) {
    let mut metrics_box = matches!(boot_metrics, BootMetrics::Enabled { .. });
    ui.horizontal_wrapped(|ui| {
        ui.checkbox(&mut metrics_box, "Boot Metrics");
        match (metrics_box, &boot_metrics) {
            (true, BootMetrics::Disabled) => *boot_metrics = BootMetrics::Enabled { timing: false },
            (false, BootMetrics::Enabled { .. }) => *boot_metrics = BootMetrics::Disabled,
            _ => {}
        }
        ui.label("Relay information about the boot process through RAM memory.");
    });
    ui.horizontal_wrapped(|ui| {
        let mut dummy = false;
        let timing_box =
            if let BootMetrics::Enabled { timing } = boot_metrics { timing } else { &mut dummy };
        ui.separator();
        ui.set_enabled(BootMetrics::timing_supported(port) && metrics_box);
        ui.checkbox(timing_box, "Timing Metrics");
        ui.label("Include boot timing as part of the boot metrics.");
    });
}
