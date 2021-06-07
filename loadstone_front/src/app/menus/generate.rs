use base64::write::EncoderWriter as Base64Encoder;
use itertools::Itertools;
use ron::ser::PrettyConfig;
use std::{fs::OpenOptions, io::Write, sync::Arc};

use anyhow::Result;
use loadstone_config::Configuration;
use reqwest_wasm::{Response, StatusCode};

use futures::future::FutureExt;

use eframe::{
    egui::{mutex::Mutex, Color32, Ui},
    epi,
};

use crate::app::utilities::download_file;

const REST_API_ENDPOINT: &str =
    "https://api.github.com/repos/absw/loadstone/actions/workflows/dispatch.yml/dispatches";

const ACTIONS_URL: &str = "https://github.com/absw/loadstone/actions";

const GITHUB_TOKEN_INSTRUCTIONS: &str = "https://docs.github.com/en/github/\
    authenticating-to-github/keeping-your-account-and-data-secure/creating-a-personal-access-token";

const LOCAL_OUTPUT_FILENAME: &str = "loadstone_config.ron";

pub fn generate<'a>(
    ui: &mut Ui,
    frame: &mut epi::Frame<'_>,
    personal_access_token_field: &mut String,
    last_request_response: &mut Arc<Mutex<Option<Result<Response, reqwest_wasm::Error>>>>,
    configuration: &Configuration,
) {
    if configuration.complete() {
        if frame.is_web() {
            ui.group(|ui| {
                generate_in_ci(
                    ui,
                    personal_access_token_field,
                    configuration,
                    last_request_response,
                );
            });
            ui.group(|ui| {
                generate_download(ui, configuration);
            });
        } else {
            generate_native(ui, configuration);
        }
    } else {
        ui.label("Provide the missing configuration to generate the loadstone binary:");
        for step in configuration.required_configuration_steps() {
            ui.colored_label(Color32::RED, format!("\u{27A1} {}.", step));
        }
    }
}

fn generate_download(ui: &mut Ui, configuration: &Configuration) {
    ui.heading("Option 2: Local");
    ui.horizontal_wrapped(|ui| {
        if ui.button("Download").clicked() {
            download_file(
                "loadstone_config.ron",
                &ron::ser::to_string_pretty(&configuration, PrettyConfig::default()).unwrap(),
            )
            .unwrap();
        }
        ui.label("Download the .ron file to build Loadstone locally.");
    });
}

fn generate_in_ci(
    ui: &mut Ui,
    personal_access_token_field: &mut String,
    configuration: &Configuration,
    last_request_response: &mut Arc<Mutex<Option<Result<Response, reqwest_wasm::Error>>>>,
) {
    ui.heading("Option 1: Github CI");
    ui.horizontal_wrapped(|ui| {
        ui.label(
            "Paste your Github PAT to trigger a Github Actions build. \
                         For instructions on how to generate a Github Personal Access Token,",
        );
        ui.hyperlink_to("visit this link.", GITHUB_TOKEN_INSTRUCTIONS);
    });
    ui.horizontal_wrapped(|ui| {
        ui.colored_label(Color32::LIGHT_BLUE, "Personal Access Token:");
        if ui.text_edit_singleline(personal_access_token_field).lost_focus() {
            let ron = ron::ser::to_string_pretty(&configuration, PrettyConfig::default())
                .unwrap_or("Invalid Configuration Supplied".into());

            generate_web(&configuration, &personal_access_token_field, &ron, last_request_response)
                .unwrap();

            personal_access_token_field.clear();
        }
    });

    match &*last_request_response.lock() {
        Some(Ok(response))
            if response.status() == StatusCode::NO_CONTENT
                || response.status() == StatusCode::ACCEPTED =>
        {
            ui.horizontal_wrapped(|ui| {
                ui.colored_label(Color32::GREEN, "Request accepted!");
                ui.label("Go to");
                ui.hyperlink_to("Loadstone's Github Actions", ACTIONS_URL);
                ui.label("to monitor your build's progress.");
            });
        }
        Some(Ok(response)) if response.status() == StatusCode::NOT_FOUND => {
            ui.colored_label(Color32::RED, "Repository not found. This likely means your Github PAT doesn't have enough rights.");
        }
        Some(Ok(response)) if response.status() == StatusCode::BAD_REQUEST => {
            ui.colored_label(
                Color32::RED,
                "Bad request. Somehow your .ron file has broken the json parser. Please download \
                             it and submit it as a bug report.",
            );
        }
        Some(_) => {
            ui.colored_label(
                Color32::RED,
                "Github Actions is not responding. Are you sure Github is up?",
            );
        }
        None => {}
    }
}

fn generate_native(ui: &mut Ui, configuration: &Configuration) {
    ui.group(|ui| {
        ui.heading("Local generation");
        ui.horizontal_wrapped(|ui| {
            if ui.button("Generate").clicked() {
                // TODO clean up unwraps
                let mut file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(LOCAL_OUTPUT_FILENAME)
                    .unwrap();
                file.write_all(
                    ron::ser::to_string_pretty(&configuration, PrettyConfig::default())
                        .unwrap()
                        .as_bytes(),
                )
                .unwrap();
            }
            ui.label("Generate a");
            ui.colored_label(Color32::LIGHT_BLUE, LOCAL_OUTPUT_FILENAME);
            ui.label("file to be used locally to build Loadstone.");
        });
    });
}

fn generate_web(
    configuration: &Configuration,
    token: &str,
    ron: &str,
    last_request_response: &mut Arc<Mutex<Option<Result<Response, reqwest_wasm::Error>>>>,
) -> Result<()> {
    let client = reqwest_wasm::Client::new();
    let cloned_response = last_request_response.clone();

    let mut auth_bytes = b"Basic ".to_vec();
    let mut encoder = Base64Encoder::new(&mut auth_bytes, base64::STANDARD);
    write!(encoder, "{}:", token).unwrap();
    drop(encoder);

    let formatted_body =format!(
            "{{\"ref\":\"staging\", \"inputs\": {{\"loadstone_configuration\":\"{}\",\"loadstone_features\":\"{}\"}}}}",
            ron.replace("\"", "\\\"").replace("\n","").replace(" ", "").replace("'", "\'"),
            configuration.required_feature_flags().collect_vec().join(","),
        );

    wasm_bindgen_futures::spawn_local(
        client
            .post(REST_API_ENDPOINT)
            .header("Accept", "application/vnd.github.v3+json")
            .header("Authorization", auth_bytes)
            .body(formatted_body)
            .send()
            .map(move |response| *cloned_response.lock() = Some(response)),
    );
    Ok(())
}
