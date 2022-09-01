//! This module manages the `Generate` dropdown menu, where
//! the Loadstone configuration as a a whole is validated and,
//! in case it's complete, gets transformed into a .ron file
//! to be sent to GithubActions or built locally.

use super::colours;
use base64::write::EncoderWriter as Base64Encoder;
use itertools::Itertools;
use ron::ser::PrettyConfig;
use std::{fs::OpenOptions, io::Write, sync::Arc};

use anyhow::Result;
use loadstone_config::Configuration;
use reqwest_wasm::{Response, StatusCode};

use futures::future::FutureExt;

use eframe::{
    egui::{mutex::Mutex, Ui},
    epi,
};

use crate::app::utilities::download_file;

const REST_API_ROOT: &str = "https://api.github.com/repos";
const REST_API_LEAF: &str = "loadstone/actions/workflows/dispatch.yml/dispatches";

const ACTIONS_URL: &str = "https://github.com/absw/loadstone/actions";

const GITHUB_TOKEN_INSTRUCTIONS: &str = "https://docs.github.com/en/github/\
    authenticating-to-github/keeping-your-account-and-data-secure/creating-a-personal-access-token";

const LOCAL_OUTPUT_FILENAME: &str = "loadstone_config.ron";

/// Renders the image generation menu.
pub fn generate<'a>(
    ui: &mut Ui,
    frame: &mut epi::Frame<'_>,
    personal_access_token_field: &mut String,
    git_ref_field: &mut String,
    git_fork_field: &mut String,
    last_request_response: &mut Arc<Mutex<Option<Result<Response, reqwest_wasm::Error>>>>,
    configuration: &Configuration,
) {
    if configuration.complete() {
        if frame.is_web() {
            generate_in_ci(
                ui,
                personal_access_token_field,
                git_ref_field,
                git_fork_field,
                configuration,
                last_request_response,
            );
            generate_download(ui, configuration);
        } else {
            generate_native(ui, configuration);
        }
    } else {
        ui.label("Provide the missing configuration to generate the loadstone binary:");
        for step in configuration.required_configuration_steps() {
            ui.colored_label(colours::error(ui), format!("\u{27A1} {}.", step));
        }
    }
}

/// Renders a link to download the finished .ron file.
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

/// Automatically triggers a Loadstone build in Github Actions. By default, this requires a
/// personal access token with write access to the main Loadstone repository, but it can
/// be pointed at different forks.
fn generate_in_ci(
    ui: &mut Ui,
    personal_access_token_field: &mut String,
    git_ref_field: &mut String,
    git_fork_field: &mut String,
    configuration: &Configuration,
    last_request_response: &mut Arc<Mutex<Option<Result<Response, reqwest_wasm::Error>>>>,
) {
    ui.heading("Option 1: Github CI");
    ui.horizontal_wrapped(|ui| {
        ui.label(
            "Paste your Github PAT to trigger a Github Actions build. \
            You must have sufficient permissions on the chosen Loadstone fork \
            to trigger workflow dispatches. \
            For instructions on how to generate a Github Personal Access Token,",
        );
        ui.hyperlink_to("visit this link.", GITHUB_TOKEN_INSTRUCTIONS);
    });
    ui.horizontal_wrapped(|ui| {
        ui.text_edit_singleline(personal_access_token_field);
        ui.colored_label(colours::info(ui), "Personal Access Token");
    });
    ui.horizontal_wrapped(|ui| {
        ui.text_edit_singleline(git_fork_field);
        ui.colored_label(colours::info(ui), "Github Fork (You must have write access)");
    });
    ui.horizontal_wrapped(|ui| {
        ui.text_edit_singleline(git_ref_field);
        ui.colored_label(colours::info(ui), "Github Ref (Branch, tag or commit hash)");
    });
    ui.horizontal_wrapped(|ui| {
        ui.set_enabled(!personal_access_token_field.is_empty());
        if ui.button("Trigger Build").clicked() {
            let ron = ron::ser::to_string_pretty(&configuration, PrettyConfig::default())
                .unwrap_or("Invalid Configuration Supplied".into());
            generate_web(
                &configuration,
                &personal_access_token_field,
                &git_ref_field,
                &git_fork_field,
                &ron,
                last_request_response,
            )
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
                ui.colored_label(colours::success(ui), "Request accepted!");
                ui.label("Go to");
                ui.hyperlink_to("Loadstone's Github Actions", ACTIONS_URL);
                ui.label("to monitor your build's progress.");
            });
        }
        Some(Ok(response)) if response.status() == StatusCode::NOT_FOUND => {
            ui.colored_label(colours::error(ui), "Repository not found. This likely means your Github PAT doesn't have enough rights.");
        }
        Some(Ok(response)) if response.status() == StatusCode::BAD_REQUEST => {
            ui.colored_label(
                colours::error(ui),
                "Bad request. Somehow your .ron file has broken the json parser. Please download \
                             it and submit it as a bug report.",
            );
        }
        Some(_) => {
            ui.colored_label(
                colours::error(ui),
                "Github Actions is not responding. Are you sure Github is up?",
            );
        }
        None => {}
    }
}

/// Generates a .ron file and saves it to the current directory. This is the
/// only available approach when running loadstone_front natively.
fn generate_native(ui: &mut Ui, configuration: &Configuration) {
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
        ui.colored_label(colours::info(ui), LOCAL_OUTPUT_FILENAME);
        ui.label("file to be used locally to build Loadstone.");
    });
}

/// Generates a loadstone image when loadstone_front is ran as a web application. Offers
/// both a download link and an automated Github Actions CI trigger.
fn generate_web(
    configuration: &Configuration,
    token: &str,
    git_ref: &str,
    git_fork: &str,
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
            "{{\"ref\":\"{}\", \"inputs\": {{\"loadstone_configuration\":\"{}\",\"loadstone_features\":\"{}\"}}}}",
            git_ref,
            ron.replace("\"", "\\\"").replace("\n",""),
            configuration.required_feature_flags().collect_vec().join(","),
        );

    wasm_bindgen_futures::spawn_local(
        client
            .post(&format!("{}/{}/{}", REST_API_ROOT, git_fork, REST_API_LEAF))
            .header("Accept", "application/vnd.github.v3+json")
            .header("Authorization", auth_bytes)
            .body(formatted_body)
            .send()
            .map(move |response| *cloned_response.lock() = Some(response)),
    );
    Ok(())
}
