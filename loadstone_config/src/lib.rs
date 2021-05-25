//! This loadstone sub-crate contains all definitions to help generate
//! final loadstone binaries.

#![feature(bool_to_option)]

use features::FeatureConfiguration;
use memory::MemoryConfiguration;
use port::Port;
use security::SecurityConfiguration;
use serde::Serialize;

mod port;
mod pins;
mod memory;
mod features;
mod security;

#[derive(Serialize)]
pub struct Configuration {
    pub port: Port,
    pub memory_configuration: MemoryConfiguration,
    pub feature_configuration: FeatureConfiguration,
    pub security_configuration: SecurityConfiguration,
}
