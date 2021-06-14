use eframe::egui;
use loadstone_config::features::UpdateSignal;

pub fn configure_update_signal(ui: &mut egui::Ui, update_signal: &mut UpdateSignal) {
    let mut enabled = matches!(update_signal, UpdateSignal::Enabled);

    ui.horizontal_wrapped(|ui| {
        ui.checkbox(&mut enabled, "Update Signal");
        ui.label("Enable update signal to control when image updates happen.");
        if enabled {
            *update_signal = UpdateSignal::Enabled;
        } else {
            *update_signal = UpdateSignal::Disabled;
        }
    });
}
