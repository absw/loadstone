#![feature(bool_to_option)]
#![allow(unused)]

use anyhow::{anyhow, Result};
use loadstone_config::{codegen::generate_modules, Configuration};
use std::fs;

fn configure_runner(target: &str) {
    println!("cargo:rerun-if-changed={}", RUNNER_TARGET_FILE);

    const RUNNER_TARGET_FILE: &str = ".cargo/.runner-target";
    fs::write(RUNNER_TARGET_FILE, target).unwrap();
}

fn main() -> Result<()> {
    process_configuration_file()?;

    #[cfg(feature = "wgm160p")]
    build_wgm160p()?;

    #[cfg(feature = "stm32f412_discovery")]
    build_stm32f412_discovery()?;

    Ok(())
}

#[allow(unused)]
fn build_wgm160p() -> Result<()> {
    configure_runner("wgm160p");
    Ok(())
}

#[allow(unused)]
fn build_stm32f412_discovery() -> Result<()> {
    configure_runner("stm32f412_discovery");
    Ok(())
}

fn process_configuration_file() -> Result<()> {
    println!("cargo:rerun-if-env-changed=LOADSTONE_CONFIG");
    println!("cargo:rerun-if-changed=loadstone_config/sample_configurations/");

    let configuration: Configuration = if let Ok(config) = std::env::var("LOADSTONE_CONFIG") {
        if config.is_empty() {
            return Ok(()); // Assuming tests
        } else {
            ron::from_str(&config)?
        }
    } else {
        panic!("\r\n\r\nBuilding Loadstone requires you supply a configuration file, \
                embedded in the `LOADSTONE_CONFIG` environment variable. \r\nTry again with \
                'LOADSTONE_CONFIG=`cat my_config.ron` cargo... \r\nIf you're just looking \
                to run unit tests, supply an empty string: 'LOADSTONE_CONFIG=\"\" cargo test`\r\n\r\n")
    };

    validate_feature_flags_against_configuration(&configuration);
    generate_modules(env!("CARGO_MANIFEST_DIR"), &configuration)?;

    Ok(())
}

#[allow(unused)]
fn validate_feature_flags_against_configuration(configuration: &Configuration) {
    let supplied_flags: Vec<_> = std::env::vars()
        .filter_map(|(k, _)| {
            k.starts_with("CARGO_FEATURE_")
                .then_some(k.strip_prefix("CARGO_FEATURE_")?.to_owned().to_lowercase())
        })
        .collect();

    let missing_flags: Vec<_> = configuration
        .feature_flags
        .iter()
        .filter(|f| !supplied_flags.contains(f))
        .cloned()
        .collect();

    if !missing_flags.is_empty() {
        panic!(
            "\r\n\r\nThe configuration file requires flags that haven't been supplied. \
            Please build again with `--features={}`\r\n\r\n",
            missing_flags.join(","),
        );
    }
}
