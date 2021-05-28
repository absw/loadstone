use eframe::egui;
use loadstone_config::{
    features::{self, Serial},
    pins,
    port::Port,
};

pub fn configure_serial(ui: &mut egui::Ui, serial: &mut Serial, port: &Port) {
    let mut serial_box = matches!(serial, Serial::Enabled { .. });
    ui.horizontal_wrapped(|ui| {
        ui.checkbox(&mut serial_box, "Serial Console");
        match (serial_box, &serial) {
            (true, Serial::Disabled) => {
                *serial = Serial::Enabled {
                    recovery_enabled: false,
                    tx_pin: pins::serial_tx(port).next().unwrap_or("Undefined").into(),
                    rx_pin: pins::serial_rx(port).next().unwrap_or("Undefined").into(),
                }
            }
            (false, Serial::Enabled { .. }) => *serial = Serial::Disabled,
            _ => {}
        };

        ui.label("Enable serial communications to retrieve information about the boot process.");
    });
    ui.scope(|ui| {
        ui.set_enabled(serial_box);
        define_serial_options(ui, serial, port);
    });
}

fn define_serial_options(ui: &mut egui::Ui, serial: &mut Serial, port: &Port) {
    ui.vertical(|ui| {
        select_tx_pins(ui, serial, port);
        select_rx_pins(ui, serial, port);
        select_recovery_mode(ui, serial, port);
    });
}

fn select_rx_pins(ui: &mut egui::Ui, serial: &mut Serial, port: &Port) {
    let mut dummy = "Unsupported".to_owned();
    let field = if let Serial::Enabled { rx_pin, .. } = serial { rx_pin } else { &mut dummy };

    ui.horizontal_wrapped(|ui| {
        ui.separator();
        ui.label("\u{2B05}");
        egui::ComboBox::from_label("Serial console input pin (RX)").selected_text(&field).show_ui(
            ui,
            |ui| {
                for pin in pins::serial_rx(port) {
                    ui.selectable_value(field, pin.to_owned(), pin);
                }
            },
        );
    });
}

fn select_tx_pins(ui: &mut egui::Ui, serial: &mut Serial, port: &Port) {
    let mut dummy = "Unsupported".to_owned();
    let field = if let Serial::Enabled { tx_pin, .. } = serial { tx_pin } else { &mut dummy };

    ui.horizontal_wrapped(|ui| {
        ui.separator();
        ui.label("\u{27A1}");
        egui::ComboBox::from_label("Serial console output pin (TX)").selected_text(&field).show_ui(
            ui,
            |ui| {
                for pin in pins::serial_tx(port) {
                    ui.selectable_value(field, pin.to_owned(), pin);
                }
            },
        );
    });
}

fn select_recovery_mode(ui: &mut egui::Ui, serial: &mut Serial, port: &Port) {
    let mut dummy = false;

    ui.horizontal_wrapped(|ui| {
        ui.set_enabled(features::Serial::supported(port));
        ui.separator();
        ui.checkbox(
            if let Serial::Enabled { recovery_enabled, .. } = serial {
                recovery_enabled
            } else {
                &mut dummy
            },
            "Serial Recovery",
        );
        ui.label("Allow recovering a device by sending a new image via XModem.");
    });
}
