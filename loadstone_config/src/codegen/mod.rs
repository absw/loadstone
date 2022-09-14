//! Generates code from parsed .ron configuration. This is where
//! concrete Loadstone modules are constructed from user configuration
//! gathered from the web app GUI.
use p256::ecdsa::VerifyingKey;
use quote::{__private::Span, quote};
use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::Path,
    process::Command,
    str::FromStr,
};
use syn::LitStr;

use crate::{
    features::{BootMetrics, Greetings, Serial, UpdateSignal},
    security::SecurityMode,
    Configuration,
};
use anyhow::Result;

use self::linker_script::generate_linker_script;
mod devices;
mod linker_script;
mod memory_map;
mod pins;

/// Transforms a `Configuration` struct into a set of source code files
/// that will be compiled into `Loadstone`. The resulting source is written
/// to src/ports/<port>/autogenerated.
pub fn generate_modules<P: AsRef<Path>>(
    loadstone_path: P,
    configuration: &Configuration,
) -> Result<()> {
    let autogenerated_folder_path = loadstone_path
        .as_ref()
        .join(format!("src/ports/{}/autogenerated", configuration.port));
    fs::create_dir(&autogenerated_folder_path).ok();
    generate_linker_script(configuration)?;
    generate_top_level_module(&autogenerated_folder_path, configuration)?;

    if std::env::var("CARGO_FEATURE_ECDSA_VERIFY").is_ok() {
        generate_key(loadstone_path, configuration)?;
    }
    memory_map::generate(
        &autogenerated_folder_path,
        &configuration.memory_configuration,
        &configuration.port,
    )?;
    pins::generate(&autogenerated_folder_path, configuration)?;
    devices::generate(&autogenerated_folder_path, configuration)?;
    Ok(())
}

/// Generates a public key file under the `src/devices/assets/` folder.
fn generate_key<P: AsRef<Path>>(loadstone_path: P, configuration: &Configuration) -> Result<()> {
    assert!(
        configuration.security_configuration.security_mode == SecurityMode::P256ECDSA,
        "Configuration mismatch: Config file requires ECDSA verification, but feature is disabled"
    );

    fs::create_dir(loadstone_path.as_ref().join("src/devices/assets/")).ok();
    let key_path = loadstone_path.as_ref().join("src/devices/assets/key.sec1");

    let key = VerifyingKey::from_str(&configuration.security_configuration.verifying_key_raw)
        .expect("Supplied public key is not valid");

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&key_path)?;
    file.write_all(key.to_encoded_point(false).as_bytes())?;
    Ok(())
}

/// Writes the top level autogenerated module, which includes a few boolean feature flags and
/// the module definitions of every autogenerated submodule.
fn generate_top_level_module<P: AsRef<Path>>(
    autogenerated_folder_path: P,
    configuration: &Configuration,
) -> Result<()> {
    let filename = autogenerated_folder_path.as_ref().join("mod.rs");
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&filename)?;

    let (serial_enabled, recovery_enabled) = if let Serial::Enabled {
        recovery_enabled, ..
    } = configuration.feature_configuration.serial
    {
        if !Serial::supported(&configuration.port) {
            panic!(
                "Serial features enabled for a port that doesn't support them: {:?}",
                configuration.port
            );
        }
        (true, recovery_enabled)
    } else {
        (false, false)
    };

    let boot_time_metrics_enabled = if let BootMetrics::Enabled { timing: true } =
        &configuration.feature_configuration.boot_metrics
    {
        if !BootMetrics::timing_supported(&configuration.port) {
            panic!(
                "Timing features enabled for a port that doesn't support them: {:?}",
                configuration.port
            );
        }
        true
    } else {
        false
    };

    let loadstone_greeting = match &configuration.feature_configuration.greetings {
        Greetings::Default => LitStr::new("-- Loadstone --", Span::call_site()),
        Greetings::Custom { loadstone, .. } => LitStr::new(loadstone, Span::call_site()),
    };
    let demo_app_greeting = match &configuration.feature_configuration.greetings {
        Greetings::Default => LitStr::new("-- Loadstone Demo App --", Span::call_site()),
        Greetings::Custom { demo, .. } => LitStr::new(demo, Span::call_site()),
    };

    let update_signal = configuration.feature_configuration.update_signal;
    let update_signal_enabled = matches!(update_signal, UpdateSignal::Enabled);

    let code = quote! {
        //! This entire module is autogenerated. Don't modify it manually!
        //! Logic for generating these files is defined under `loadstone_config/src/codegen/`
        pub mod memory_map;
        pub mod pin_configuration;
        pub mod devices;

        #[allow(unused)]
        pub const SERIAL_ENABLED: bool = #serial_enabled;
        #[allow(unused)]
        pub const RECOVERY_ENABLED: bool = #recovery_enabled;
        #[allow(unused)]
        pub const BOOT_TIME_METRICS_ENABLED: bool = #boot_time_metrics_enabled;
        #[allow(unused)]
        pub const LOADSTONE_GREETING: &str = #loadstone_greeting;
        #[allow(unused)]
        pub const DEMO_APP_GREETING: &str = #demo_app_greeting;
        #[allow(unused)]
        pub const UPDATE_SIGNAL_ENABLED: bool = #update_signal_enabled;
    };

    file.write_all(format!("{}", code).as_bytes())?;
    prettify_file(filename).ok();
    Ok(())
}

fn prettify_file<P: AsRef<Path>>(path: P) -> io::Result<()> {
    Command::new("rustfmt").arg(path.as_ref()).spawn()?.wait()?;
    Ok(())
}
