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

use features::FeatureConfiguration;
use memory::MemoryConfiguration;
use port::Port;
use security::{SecurityConfiguration, SecurityMode};
use serde::{Serialize, Deserialize};

pub mod port;
pub mod pins;
pub mod memory;
pub mod features;
pub mod security;

#[derive(Serialize, Deserialize, Default)]
pub struct Configuration {
    pub port: Port,
    pub memory_configuration: MemoryConfiguration,
    pub feature_configuration: FeatureConfiguration,
    pub security_configuration: SecurityConfiguration,
}

impl Configuration {
    pub fn new(
        port: Port,
        memory_configuration: MemoryConfiguration,
        feature_configuration: FeatureConfiguration,
        security_configuration: SecurityConfiguration,
    ) -> Self {
        Self { port, memory_configuration, feature_configuration, security_configuration }
    }
}

impl Configuration {
    pub fn complete(&self) -> bool { self.required_configuration_steps().count() == 0 }

    pub fn required_configuration_steps(&self) -> impl Iterator<Item = RequiredConfigurationStep> {
        #[rustfmt::skip]
        IntoIter::new([
            // Family must always be defined
            self.port.family.is_none().then_some(RequiredConfigurationStep::Family),

            // Subfamily is optional depending on the granularity of the internal flash port
            (self.port.family.is_some() && self.port.subfamily.is_none() && memory::internal_flash(&self.port).is_none())
                .then_some(RequiredConfigurationStep::Subfamily),

            // Board is optional depending on the granularity of the internal flash port
            (self.port.subfamily.is_some() && self.port.board.is_none() && memory::internal_flash(&self.port).is_none())
                .then_some(RequiredConfigurationStep::Board),

            (self.feature_configuration.serial.enabled
                && self.feature_configuration.serial.tx_pin.is_none())
                .then_some(RequiredConfigurationStep::SerialTxPin),

            (self.feature_configuration.serial.enabled
                && self.feature_configuration.serial.rx_pin.is_none())
                .then_some(RequiredConfigurationStep::SerialRxPin),

            self.memory_configuration.internal_memory_map.bootable_index.is_none()
                .then_some(RequiredConfigurationStep::BootableBank),

            (self.security_configuration.security_mode == SecurityMode::P256ECDSA
                && self.security_configuration.verifying_key_raw.is_empty())
                .then_some(RequiredConfigurationStep::PublicKey),

        ])
        .flatten()
    }
}

pub enum RequiredConfigurationStep {
    Family,
    Subfamily,
    Board,
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
            RequiredConfigurationStep::Family => "[Target] Specify target MCU family",
            RequiredConfigurationStep::Subfamily => "[Target] Specify target MCU subfamily",
            RequiredConfigurationStep::Board => "[Target] Specify target board",
            RequiredConfigurationStep::SerialTxPin => "[Features] Define Serial Tx pin",
            RequiredConfigurationStep::SerialRxPin => "[Features] Define Serial Rx pin",
            RequiredConfigurationStep::BootableBank => "[Memory Map] Define a bootable bank",
        })
    }
}

