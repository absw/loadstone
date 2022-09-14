use std::{fs::OpenOptions, io::Write};

use crate::{port::LinkerScriptConstants, Configuration};
use anyhow::{anyhow, Result};

/// Generates the linker script `memory.x`, which describes the amount and location
/// of flash and RAM memory available to a particular Loadstone instance.
pub fn generate_linker_script(configuration: &Configuration) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("memory.x")?;

    #[allow(unused_mut)]
    let mut constants = configuration
        .port
        .linker_script_constants()
        .ok_or_else(|| anyhow!("Current board doesn't have linker script constants defined."))?;

    if std::env::var("CARGO_FEATURE_RELOCATE_TO_BOOTABLE_BANK").is_ok() {
        relocate_to_bootable_bank(&mut constants, configuration)?;
    }

    write!(
        file,
        "MEMORY\n\
         {{\n\
             FLASH : ORIGIN = 0x{:08X}, LENGTH = {}K\n\
             RAM : ORIGIN = 0x{:08X}, LENGTH = {}K\n\
         }}\n",
        constants.flash.origin,
        constants.flash.size / 1024,
        constants.ram.origin,
        constants.ram.size / 1024,
    )?;

    Ok(())
}

#[allow(unused)]
fn relocate_to_bootable_bank(
    constants: &mut LinkerScriptConstants,
    configuration: &Configuration,
) -> Result<()> {
    let bootable_address = configuration
        .memory_configuration
        .bootable_address()
        .ok_or_else(|| {
            anyhow!("Impossible to relocate: bootable bank is undefined in configuration file.")
        })?;
    let offset = bootable_address - constants.flash.origin;
    constants.flash.size = constants.flash.size.saturating_sub(offset as usize);
    constants.flash.origin = bootable_address;
    Ok(())
}
