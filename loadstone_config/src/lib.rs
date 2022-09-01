//! This loadstone sub-crate contains all definitions to help generate
//! final loadstone binaries.
//!
//! NOTE: This code is not included anywhere from Loadstone itself! This
//! is a dependency of the Loadstone **build script**. The build script
//! uses this dependency to help generate the code that Loadstone includes
//! (things like feature flags, memory map configuration, etc).

use std::fmt::Display;

use features::{BootMetrics, FeatureConfiguration, Serial};
use memory::{external_flash, MemoryConfiguration};
use port::Port;
use security::{SecurityConfiguration, SecurityMode};
use serde::{Deserialize, Serialize};

pub mod port;
pub mod pins;
pub mod memory;
pub mod features;
pub mod security;
pub mod codegen;

#[derive(Serialize, Deserialize, Default, Debug)]
/// Defines all configuration for a "codegen" loadstone port. This struct
/// is meant to be modified live by the `loadstone_front` GUI, then serialized
/// into a .ron file, which will be read by the loadstone `build.rs` script
/// and turned into the port source.
pub struct Configuration {
    /// The target chip, usually defined at the chip subfamily level (e.g stm32f412).
    pub port: Port,
    /// Internal and external flash configuration, including firmware image
    /// banks and bank sizes.
    pub memory_configuration: MemoryConfiguration,
    /// Miscellaneous features such as serial communication or boot metrics.
    pub feature_configuration: FeatureConfiguration,
    /// Image authenticity, integrity and (potentially) secrecy options (ECDSA, CRC, etc).
    pub security_configuration: SecurityConfiguration,
}

impl Configuration {
    /// True if the configuration is comprehensive enough to generate a loadstone binary.
    pub fn complete(&self) -> bool { self.required_configuration_steps().count() == 0 }

    /// Returns an iterator over the feature flags that will be necessary to compile loadstone
    /// when using this configuration struct.
    pub fn required_feature_flags(&self) -> impl Iterator<Item = &'static str> {
        let mut flags = vec![];
        match self.port {
            Port::Stm32F412 => flags.push("stm32f412"),
            Port::Wgm160P => flags.push("wgm160p"),
            Port::Max32631 => flags.push("max32631"),
        };

        if self.security_configuration.security_mode == SecurityMode::P256ECDSA {
            flags.push("ecdsa-verify");
        };

        flags.into_iter()
    }

    /// Missing configuration steps to have enough information to generate a loadstone binary.
    pub fn required_configuration_steps(&self) -> impl Iterator<Item = RequiredConfigurationStep> {
        IntoIterator::into_iter([
            self.memory_configuration.internal_memory_map.bootable_index.is_none()
                .then_some(RequiredConfigurationStep::BootableBank),

            (self.security_configuration.security_mode == SecurityMode::P256ECDSA
                && self.security_configuration.verifying_key_raw.is_empty())
                .then_some(RequiredConfigurationStep::PublicKey),

        ])
        .flatten()
    }

    /// Cleans up the configuration, enforcing all internal invariants.
    // TODO replace with typestates / type safety wherever possible, by adjusting the loadstone
    // front app to match.
    pub fn cleanup(&mut self) {
        if !features::Serial::supported(&self.port) {
            self.feature_configuration.serial = Serial::Disabled;
        }

        self.memory_configuration.internal_memory_map.banks.truncate(u8::MAX as usize);
        let max_external_banks = (u8::MAX as usize)
            - self.memory_configuration.internal_memory_map.banks.len();
        self.memory_configuration.external_memory_map.banks.truncate(max_external_banks);

        if !features::BootMetrics::timing_supported(&self.port) {
            if let BootMetrics::Enabled{timing} = &mut self.feature_configuration.boot_metrics {
                *timing = false
            }
        }

        if !matches!(self.security_configuration.security_mode, SecurityMode::P256ECDSA) {
            self.security_configuration.verifying_key_raw.clear();
        }

        if let Some(golden_index) = self.memory_configuration.golden_index {
            let bank_count = self.memory_configuration.internal_memory_map.banks.len()
                + self.memory_configuration.external_memory_map.banks.len();
            if golden_index >= bank_count {
                self.memory_configuration.golden_index = None;
            }
        }

        if !external_flash(&self.port).any(|f| Some(f) == self.memory_configuration.external_flash)
        {
            self.memory_configuration.external_flash = None;
        }

        if self.memory_configuration.external_flash.is_none() {
            self.memory_configuration.external_memory_map.banks.clear();
        }
    }
}

/// Configuration steps that may be required to properly define a loadstone binary.
pub enum RequiredConfigurationStep {
    PublicKey,
    SerialTxPin,
    SerialRxPin,
    BootableBank,
}

impl Display for RequiredConfigurationStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            RequiredConfigurationStep::PublicKey => {
                "[Security] Provide P256 ECDSA public key or enable CRC32 mode"
            }
            RequiredConfigurationStep::SerialTxPin => "[Features] Define Serial Tx pin",
            RequiredConfigurationStep::SerialRxPin => "[Features] Define Serial Rx pin",
            RequiredConfigurationStep::BootableBank => "[Memory Map] Define a bootable bank",
        })
    }
}
