use serde::{Deserialize, Serialize};

use crate::port::Port;

#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub struct FeatureConfiguration {
    pub serial: Serial,
    pub boot_metrics: BootMetrics,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum BootMetrics {
    Enabled { timing: bool },
    Disabled,
}

impl Default for BootMetrics {
    fn default() -> Self { Self::Disabled }
}

impl BootMetrics {
    pub fn timing_supported(port: &Port) -> bool {
        match port {
            Port::Stm32F412 => true,
            Port::Wgm160P => false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Serial {
    Enabled { recovery_enabled: bool, tx_pin: String, rx_pin: String },
    Disabled,
}

impl Default for Serial {
    fn default() -> Self { Self::Disabled }
}

impl Serial {
    pub fn supported(port: &Port) -> bool {
        match port {
            Port::Stm32F412 => true,
            Port::Wgm160P => false,
        }
    }
    pub fn enabled(&self) -> bool { matches!(self, Serial::Enabled {..}) }
}
