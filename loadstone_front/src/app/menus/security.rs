use super::colours;
use eframe::egui::{self, Button};
use loadstone_config::security::SecurityMode;
use p256::ecdsa::VerifyingKey;
use std::str::FromStr;

/// Renders the menu to configure security options (at the moment,
/// `CRC` and `ECDSA` image verification.
pub fn configure_security(
    ui: &mut egui::Ui,
    security_mode: &mut SecurityMode,
    verifying_key_raw: &mut String,
    verifying_key_text_field: &mut String,
) {
    ui.horizontal_wrapped(|ui| {
        ui.radio_value(security_mode, SecurityMode::P256ECDSA, "Enable P256 ECDSA mode.")
            .on_hover_text("Enable P256 ECDSA signature verification.");
        ui.radio_value(security_mode, SecurityMode::Crc, "Enable CRC32 mode.")
            .on_hover_text("Disable ECDSA verification in favor of IEEE CRC32");
    });

    match security_mode {
        SecurityMode::Crc => {
            ui.colored_label(
                colours::warning(ui),
                "WARNING: Disabling ECDSA Image Verification replaces cryptographic \
                signatures with insecure CRC. This removes the guarantee of image authenticity.",
            );
        }
        SecurityMode::P256ECDSA => {
            ui.label("P256 ECDSA Public Key");

            if !verifying_key_raw.is_empty() {
                ui.horizontal_wrapped(|ui| {
                    ui.colored_label(colours::success(ui), "\u{1F5DD} Valid Key Supplied");
                    if ui
                        .add(Button::new("Delete").text_color(colours::error(ui)).small())
                        .clicked()
                    {
                        verifying_key_raw.clear();
                    };
                });
            } else {
                if ui.text_edit_multiline(verifying_key_text_field).lost_focus() {
                    // Preprocess the key to ensure spaces are maintained
                    *verifying_key_text_field = verifying_key_text_field
                        .replace("-----BEGIN PUBLIC KEY----- ", "-----BEGIN PUBLIC KEY-----\n")
                        .replace(" -----END PUBLIC KEY-----", "\n-----END PUBLIC KEY-----");
                    if VerifyingKey::from_str(&verifying_key_text_field).is_ok() {
                        *verifying_key_raw = verifying_key_text_field.clone();
                    } else {
                        *verifying_key_text_field = String::new();
                    }
                }

                ui.label("Please paste a valid public key in PEM format");
            }
        }
    }
}
