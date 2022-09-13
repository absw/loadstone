use std::sync::Arc;

use self::menus::{
    configure_boot_metrics, memory_map::configure_memory_map, security::configure_security,
    select_port,
};

use crate::app::menus::{
    configure_custom_greetings, generate, serial::configure_serial,
    update_signal::configure_update_signal,
};
use eframe::{
    egui::{self, mutex::Mutex, ScrollArea, TextStyle},
    epi,
};
const GIT_VERSION: &str = git_version::git_version!();

use loadstone_config::{features::Serial, pins, Configuration};
use reqwest_wasm::Response;

mod menus;
mod utilities;

/// Contains all persistent information required to render the loadstone web app
/// options, and therefore fully define a Loadstone port. It wraps
/// loadstone_config's `Configuration` struct, which can be serialized into a .ron
/// file to be later consumed by Loadstone when generating code.
// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
pub struct LoadstoneApp {
    configuration: Configuration,
    verifying_key_text_field: String,
    personal_access_token_field: String,
    git_fork_field: String,
    git_ref_field: String,
    /// This complicated type exists to hold the last response to our outgoing POST
    /// requests to github actions. It must be thread safe as responses are received
    /// in a separate context.
    last_request_response: Arc<Mutex<Option<Result<Response, reqwest_wasm::Error>>>>,
}

impl Default for LoadstoneApp {
    fn default() -> Self {
        Self {
            configuration: Default::default(),
            verifying_key_text_field: Default::default(),
            personal_access_token_field: Default::default(),
            git_ref_field: "staging".into(),
            git_fork_field: "absw".into(),
            last_request_response: Arc::new(Mutex::new(None)),
        }
    }
}

impl epi::App for LoadstoneApp {
    fn name(&self) -> &str {
        "Loadstone Builder"
    }

    /// Called by the framework to load old app state (if any).
    #[cfg(feature = "persistence")]
    fn load(&mut self, storage: &dyn epi::Storage) {
        *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
    }

    /// Called by the frame work to save state before shutdown.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        let LoadstoneApp {
            configuration,
            verifying_key_text_field,
            personal_access_token_field,
            last_request_response,
            git_ref_field,
            git_fork_field,
        } = self;
        configuration.cleanup();

        ctx.set_style(egui::Style {
            spacing: egui::style::Spacing {
                slider_width: 200.0,
                text_edit_width: 560.0,
                scroll_bar_width: 12.0,
                ..Default::default()
            },
            ..Default::default()
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ScrollArea::auto_sized().show(ui, |ui| {
                ui.heading(format!(
                    "Loadstone builder [{}-{}] ",
                    env!("CARGO_PKG_VERSION"),
                    GIT_VERSION
                ));
                ui.separator();
                select_port(ui, &mut configuration.port);
                ui.separator();
                egui::CollapsingHeader::new("Features")
                    .text_style(TextStyle::Heading)
                    .show(ui, |ui| {
                        ui.label(
                            "Greyed out features are unsupported in the current configuration.",
                        );
                        ui.set_enabled(
                            Serial::supported(&configuration.port)
                                && pins::serial_tx(&configuration.port).count() > 0
                                && pins::serial_rx(&configuration.port).count() > 0,
                        );
                        configure_serial(
                            ui,
                            &mut configuration.feature_configuration.serial,
                            &configuration.port,
                        );
                        configure_boot_metrics(
                            ui,
                            &mut configuration.feature_configuration.boot_metrics,
                            &configuration.port,
                        );
                        configure_custom_greetings(
                            ui,
                            &mut configuration.feature_configuration.greetings,
                        );
                        configure_update_signal(
                            ui,
                            &mut configuration.feature_configuration.update_signal,
                        );
                    });
                egui::CollapsingHeader::new("Memory map")
                    .text_style(TextStyle::Heading)
                    .show(ui, |ui| {
                        configure_memory_map(
                            ui,
                            &mut configuration.memory_configuration.internal_memory_map,
                            &mut configuration.memory_configuration.external_memory_map,
                            &mut configuration.memory_configuration.external_flash,
                            &mut configuration.memory_configuration.golden_index,
                            &configuration.port,
                        );
                    });
                egui::CollapsingHeader::new("Security")
                    .text_style(TextStyle::Heading)
                    .show(ui, |ui| {
                        configure_security(
                            ui,
                            &mut configuration.security_configuration.security_mode,
                            &mut configuration.security_configuration.verifying_key_raw,
                            verifying_key_text_field,
                        );
                    });
                egui::CollapsingHeader::new("Generate")
                    .text_style(TextStyle::Heading)
                    .show(ui, |ui| {
                        generate::generate(
                            ui,
                            frame,
                            personal_access_token_field,
                            git_ref_field,
                            git_fork_field,
                            last_request_response,
                            configuration,
                        );
                    });
            });
        });
    }
}