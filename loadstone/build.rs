use anyhow::Result;
use loadstone_config::{codegen::generate_modules, security::SecurityMode, Configuration};
use std::fs;

fn configure_runner(target: &str) {
    println!("cargo:rerun-if-changed={}", RUNNER_TARGET_FILE);

    const RUNNER_TARGET_FILE: &str = ".cargo/.runner-target";
    fs::write(RUNNER_TARGET_FILE, target).unwrap();
}

fn main() -> Result<()> { process_configuration_file() }

fn process_configuration_file() -> Result<()> {
    println!("cargo:rerun-if-env-changed=LOADSTONE_CONFIG");

    let configuration: Configuration = if let Ok(config) = std::env::var("LOADSTONE_CONFIG") {
        if config.is_empty() {
            return Ok(()); // Assuming tests
        } else {
            ron::from_str(&config)?
        }
    } else {
        panic!(
            "\n\nBuilding Loadstone requires you supply a configuration file, \
                embedded in the `LOADSTONE_CONFIG` environment variable. \nTry again with \
                'LOADSTONE_CONFIG=`cat my_config.ron` cargo... \nIf you're just looking \
                to run unit tests, or to build a port that does not require any code \
                generation (manual port), supply an empty string:
                'LOADSTONE_CONFIG=\"\" cargo...`\n\n"
        )
    };

    validate_feature_flags_against_configuration(&configuration);
    generate_modules(env!("CARGO_MANIFEST_DIR"), &configuration)?;
    configure_runner(&configuration.port.to_string());

    Ok(())
}

fn validate_feature_flags_against_configuration(configuration: &Configuration) {
    let supplied_flags: Vec<_> = std::env::vars()
        .filter_map(|(k, _)| {
            k.starts_with("CARGO_FEATURE_")
                .then_some(k.strip_prefix("CARGO_FEATURE_")?.to_owned().to_lowercase())
        })
        .collect();

    let missing_flags: Vec<_> = configuration
        .required_feature_flags()
        .map(|s| s.replace('-', "_"))
        .filter(|f| !&supplied_flags.contains(&(*f).to_owned()))
        .collect();

    if configuration.security_configuration.security_mode != SecurityMode::P256ECDSA
        && supplied_flags.contains(&"ecdsa_verify".to_owned())
    {
        panic!("Configuration mismatch. Configuration file does not specify ECDSA security mode, \
                but the `ecdsa-verify` flag was supplied. Try again without `ecdsa-verify` for CRC mode.");
    }

    if !missing_flags.is_empty() {
        panic!(
            "\n\nThe configuration file requires flags that haven't been supplied. \
            Please build again with `--features={}`\n\n",
            missing_flags.join(",").replace('_', "-"),
        );
    }
}
