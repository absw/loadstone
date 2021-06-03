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

use features::{FeatureConfiguration, Serial};
use memory::{external_flash, MemoryConfiguration};
use port::{board::STM32F412, Port};
use security::{SecurityConfiguration, SecurityMode};
use serde::{Deserialize, Serialize};

use crate::port::board::WGM160P;

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
    pub feature_flags: Vec<String>,
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
        if !self.port.family.as_ref().map_or(false, |f| f.contains(self.port.subfamily.as_ref())) {
            self.port.subfamily = None;
        }
        if !self.port.subfamily.as_ref().map_or(false, |f| f.contains(self.port.board.as_ref())) {
            self.port.board = None;
        }

        if !features::Serial::supported(&self.port) {
            self.feature_configuration.serial = Serial::Disabled;
        }

        if !external_flash(&self.port).any(|f| Some(f) == self.memory_configuration.external_flash)
        {
            self.memory_configuration.external_flash = None;
        }

        if self.memory_configuration.external_flash.is_none() {
            self.memory_configuration.external_memory_map.banks.clear();
        }

        match self.port.board_name() {
            name if name == STM32F412 => self.feature_flags = vec!["stm34f412_discovery".into()],
            name if name == WGM160P => self.feature_flags = vec!["wgm160p".into()],
            _ => {}
        }
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
