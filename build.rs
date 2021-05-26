use anyhow::Result;
use loadstone_config::{
    codegen::generate_modules,
    port::{family, subfamily},
    Configuration,
};
use std::{
    fs::{self, File},
    io::{BufReader, Read},
};

fn configure_memory_x(file: &str) {
    let filename = format!("memory/{}", file);

    println!("cargo:rerun-if-changed=memory.x");
    println!("cargo:rerun-if-changed={}", &filename);

    fs::copy(&filename, "memory.x").unwrap();
}

fn configure_runner(target: &str) {
    println!("cargo:rerun-if-changed={}", RUNNER_TARGET_FILE);

    const RUNNER_TARGET_FILE: &str = ".cargo/.runner-target";
    fs::write(RUNNER_TARGET_FILE, target).unwrap();
}

#[cfg(feature = "wgm160p")]
fn main() -> Result<()> {
    process_configuration_file()?;
    configure_memory_x("wgm160p.x");
    configure_runner("wgm160p");
    Ok(())
}

#[cfg(feature = "stm32f412_discovery")]
fn main() -> Result<()> {
    println!("cargo:rerun-if-env-changed=LOADSTONE_USE_ALT_MEMORY");

    let use_alt_memory = match option_env!("LOADSTONE_USE_ALT_MEMORY") {
        None => false,
        Some("0") => false,
        Some("1") => true,
        _ => panic!("LOADSTONE_USE_ALT_MEMORY must be 0, 1 or undefined."),
    };

    let memory_file =
        if use_alt_memory { "stm32f412_discovery.alt.x" } else { "stm32f412_discovery.x" };

    configure_memory_x(memory_file);
    configure_runner("stm32f412_discovery");
    process_configuration_file()?;
    Ok(())
}

#[cfg(feature = "stm32f412_discovery")]
const DEFAULT_CONFIG_FILENAME: &str = "stm32f412_discovery_default_config.ron";

#[cfg(feature = "wgm160p")]
const DEFAULT_CONFIG_FILENAME: &str = "wgm160p_default_config.ron";

#[cfg(not(any(feature = "stm32f412_discovery", feature = "wgm160p")))]
const DEFAULT_CONFIG_FILENAME: &str = "";

fn process_configuration_file() -> Result<()> {
    println!("cargo:rerun-if-env-changed=LOADSTONE_CONFIG");
    println!(
        "cargo:rerun-if-changed=./loadstone_config/sample_configurations/{}",
        DEFAULT_CONFIG_FILENAME
    );

    let filename = if let Some(filename) = option_env!("LOADSTONE_CONFIG") {
        filename.into()
    } else {
        // This will eventually be removed, as defaults for something
        // as complex as a bootloader aren't really meaningful.
        // It's currently useful for testing however.
        format!("./loadstone_config/sample_configurations/{}", DEFAULT_CONFIG_FILENAME)
    };

    let file = File::open(filename)?;
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;
    let configuration: Configuration = ron::from_str(&contents)?;
    validate_feature_flags_against_configuration(&configuration);
    generate_modules("./", &configuration)?;

    Ok(())
}

fn validate_feature_flags_against_configuration(configuration: &Configuration) {
    #[cfg(feature = "stm32_any")]
    assert_eq!(configuration.port.family_name(), family::STM32,
        "Mismatching MCU family in configuration file. Features require {}, configuration requires {}",
         configuration.port.family_name(),
         family::STM32);

    #[cfg(feature = "stm32f412")]
    assert_eq!(configuration.port.subfamily_name(), subfamily::STM32F4,
        "Mismatching MCU subfamily in configuration file. Features require {}, configuration requires {}",
         configuration.port.subfamily_name(),
         subfamily::STM32F4);

    #[cfg(feature = "wgm160p")]
    assert_eq!(configuration.port.board_name(), board::WGM160P,
        "Mismatching MCU family in configuration file. Features require {}, configuration requires {}",
         configuration.port.board_name(),
         board::WGM160P);

    #[cfg(feature = "serial")]
    assert!(configuration.feature_configuration.serial.enabled,
        "Configuration mismatch. Feature flags require `serial`, but it is disabled in the configuration file");

    #[cfg(feature = "serial-recovery")]
    assert!(configuration.feature_configuration.serial.recovery_enabled,
        "Configuration mismatch. Feature flags require `serial recovery`, but it is disabled in the configuration file");

    #[cfg(feature = "boot-time-metrics")]
    assert!(configuration.feature_configuration.boot_metrics.enabled && configuration.feature_configuration.boot_metrics.timing_enabled,
        "Configuration mismatch. Feature flags require `boot timing`, but it is disabled in the configuration file");
}
