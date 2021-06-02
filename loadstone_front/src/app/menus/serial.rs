use eframe::egui;
use itertools::Itertools;
use loadstone_config::{features::{self, Serial}, pins::{self, Peripheral, Pin}, port::Port};

pub fn configure_serial(ui: &mut egui::Ui, serial: &mut Serial, port: &Port) {
    let mut available_peripherals =
        pins::serial_tx(port).chain(pins::serial_rx(port)).map(|p| p.peripheral).collect_vec();
    available_peripherals.sort();
    available_peripherals.dedup();

    let first_valid_tx_pin = || {
        pins::serial_tx(port)
            .find_map(|p| (p.peripheral == available_peripherals[0]).then_some(p))
            .unwrap()
    };

    let first_valid_rx_pin = || {
        pins::serial_tx(port)
            .find_map(|p| (p.peripheral == available_peripherals[0]).then_some(p))
            .unwrap()
    };

    let mut serial_box = matches!(serial, Serial::Enabled { .. });
    ui.horizontal_wrapped(|ui| {
        ui.checkbox(&mut serial_box, "Serial Console");
        match (serial_box, &serial) {
            (true, Serial::Disabled) => {
                *serial = Serial::Enabled {
                    recovery_enabled: false,
                    tx_pin: first_valid_tx_pin(),
                    rx_pin: first_valid_rx_pin(),
                }
            }
            (false, Serial::Enabled { .. }) => *serial = Serial::Disabled,
            _ => {}
        };

        ui.label("Enable serial communications to retrieve information about the boot process.");
    });
    if let Serial::Enabled { recovery_enabled, tx_pin, rx_pin } = serial {
        define_serial_options(
            ui,
            port,
            recovery_enabled,
            tx_pin,
            rx_pin,
            available_peripherals.iter().cloned(),
        );
    }
}

fn define_serial_options(
    ui: &mut egui::Ui,
    port: &Port,
    recovery_enabled: &mut bool,
    tx_pin: &mut Pin,
    rx_pin: &mut Pin,
    available_peripherals: impl Iterator<Item = Peripheral>,
) {
    ui.vertical(|ui| {
        select_peripheral(ui, port, tx_pin, rx_pin, available_peripherals);
        select_tx_pins(ui, tx_pin, port);
        select_rx_pins(ui, rx_pin, port);
        select_recovery_mode(ui, recovery_enabled, port);
    });
}

fn select_peripheral(
    ui: &mut egui::Ui,
    port: &Port,
    tx_pin: &mut Pin,
    rx_pin: &mut Pin,
    available_peripherals: impl Iterator<Item = Peripheral>,
) {
    let mut inferred_peripheral = tx_pin.peripheral.clone();

    ui.horizontal_wrapped(|ui| {
        egui::ComboBox::from_label("Serial Peripheral")
            .selected_text(&inferred_peripheral)
            .show_ui(ui, |ui| {
                for peripheral in available_peripherals {
                    ui.selectable_value(&mut inferred_peripheral, peripheral.clone(), peripheral);
                }
            });
    });

    let first_valid_tx =
        |peripheral| pins::serial_tx(port).find_map(|p| (&p.peripheral == peripheral).then_some(p)).unwrap();
    let first_valid_rx =
        |peripheral| pins::serial_rx(port).find_map(|p| (&p.peripheral == peripheral).then_some(p)).unwrap();

    if tx_pin.peripheral != inferred_peripheral {
        *tx_pin = first_valid_tx(&inferred_peripheral);
    }
    if rx_pin.peripheral != inferred_peripheral {
        *rx_pin = first_valid_rx(&inferred_peripheral);
    }
}

fn select_rx_pins(ui: &mut egui::Ui, rx_pin: &mut Pin, port: &Port) {
    ui.horizontal_wrapped(|ui| {
        ui.separator();
        ui.label("\u{2B05}");
        egui::ComboBox::from_label("Serial console input pin (RX)")
            .selected_text(rx_pin.to_string())
            .show_ui(ui, |ui| {
                let peripheral = rx_pin.peripheral.clone();
                for pin in pins::serial_rx(port).filter(|pin| pin.peripheral == peripheral) {
                    ui.selectable_value(rx_pin, pin.clone(), pin);
                }
            });
    });
}

fn select_tx_pins(ui: &mut egui::Ui, tx_pin: &mut Pin, port: &Port) {
    ui.horizontal_wrapped(|ui| {
        ui.separator();
        ui.label("\u{27A1}");
        egui::ComboBox::from_label("Serial console output pin (TX)")
            .selected_text(tx_pin.to_string())
            .show_ui(ui, |ui| {
                let peripheral = tx_pin.peripheral.clone();
                for pin in pins::serial_tx(port).filter(|pin| pin.peripheral == peripheral) {
                    ui.selectable_value(tx_pin, pin.clone(), pin);
                }
            });
    });
}

fn select_recovery_mode(ui: &mut egui::Ui, recovery_enabled: &mut bool, port: &Port) {
    ui.horizontal_wrapped(|ui| {
        ui.set_enabled(features::Serial::supported(port));
        ui.separator();
        ui.checkbox(recovery_enabled, "Serial Recovery");
        ui.label("Allow recovering a device by sending a new image via XModem.");
    });
}
