use eframe::egui;
use enum_iterator::IntoEnumIterator;
use loadstone_config::{
    features::{BootMetrics, Greetings},
    port::Port,
};

pub mod memory_map;
pub mod security;
pub mod generate;
pub mod update_signal;
pub mod serial;

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

pub fn configure_custom_greetings(ui: &mut egui::Ui, greetings: &mut Greetings) {
    let mut greetings_box = matches!(greetings, Greetings::Custom { .. });
    let loadstone_with_version = || {
        format!(
            "-- Loadstone [{}-{}] --",
            env!("CARGO_PKG_VERSION"),
            git_version::git_version!()
        )
    };
    let demo_with_version = || {
        format!(
            "-- Loadstone Demo App [{}-{}] --",
            env!("CARGO_PKG_VERSION"),
            git_version::git_version!()
        )
    };
    ui.horizontal_wrapped(|ui| {
        ui.checkbox(&mut greetings_box, "Custom Greetings");
        match (greetings_box, &greetings) {
            (true, Greetings::Default) => {
                *greetings = Greetings::Custom {
                    loadstone: loadstone_with_version().into(),
                    demo: demo_with_version().into(),
                }
            }
            (false, Greetings::Custom { .. }) => *greetings = Greetings::Default,
            _ => {}
        }
        ui.label("Select custom greetings for Loadstone and the demo application.");
    });

    if let Greetings::Custom { loadstone, demo } = greetings {
        ui.horizontal_wrapped(|ui| {
            ui.text_edit_singleline(loadstone.to_mut());
            ui.label("Custom greeting when booting Loadstone.");
        });
        ui.horizontal_wrapped(|ui| {
            ui.text_edit_singleline(demo.to_mut());
            ui.label("Custom greeting when booting the demo application.");
        });
    }
}
