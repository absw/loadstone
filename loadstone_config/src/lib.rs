//! This loadstone sub-crate contains all definitions to help generate
//! final loadstone binaries.
//!
//! NOTE: This code is not included anywhere from Loadstone itself! This
//! is a dependency of the Loadstone **build script**. The build script
//! uses this dependency to help generate the code that Loadstone includes
//! (things like feature flags, memory map configuration, etc).

#![feature(stmt_expr_attributes)]
#![feature(bool_to_option)]

use std::{array::IntoIter, fmt::Display};

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
pub struct Configuration {
    pub port: Port,
    pub memory_configuration: MemoryConfiguration,
    pub feature_configuration: FeatureConfiguration,
    pub security_configuration: SecurityConfiguration,
}

impl Configuration {
    pub fn complete(&self) -> bool { self.required_configuration_steps().count() == 0 }

    pub fn required_feature_flags(&self) -> impl Iterator<Item = &'static str> {
        let mut flags = vec![];
        match self.port {
            Port::Stm32F412 => flags.push("stm32f412"),
            Port::Wgm160P => flags.push("wgm160p"),
        };

        if self.security_configuration.security_mode == SecurityMode::P256ECDSA {
            flags.push("ecdsa-verify");
        };

        flags.into_iter()
    }

    pub fn required_configuration_steps(&self) -> impl Iterator<Item = RequiredConfigurationStep> {
        #[rustfmt::skip]
        IntoIter::new([
            self.memory_configuration.internal_memory_map.bootable_index.is_none()
                .then_some(RequiredConfigurationStep::BootableBank),

            (self.security_configuration.security_mode == SecurityMode::P256ECDSA
                && self.security_configuration.verifying_key_raw.is_empty())
                .then_some(RequiredConfigurationStep::PublicKey),

        ])
        .flatten()
    }

    // Enforces all internal invariants.
    // TODO replace with typestates / type safety wherever possible, by adjusting the loadstone
    // front app to match
    pub fn cleanup(&mut self) {
        if !features::Serial::supported(&self.port) {
            self.feature_configuration.serial = Serial::Disabled;
        }

        if !features::BootMetrics::timing_supported(&self.port) {
            if let BootMetrics::Enabled{timing} = &mut self.feature_configuration.boot_metrics {
                *timing = false
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
