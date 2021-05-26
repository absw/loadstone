use crate::port::{family, subfamily, Port};
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub struct FeatureConfiguration {
    pub serial: Serial,
    pub boot_metrics: BootMetrics,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BootMetrics {
    pub enabled: bool,
    pub timing_enabled: bool,
}

impl BootMetrics {
    pub fn timing_supported(port: &Port) -> bool { port.family_name() == family::STM32 }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Serial {
    pub enabled: bool,
    pub recovery_enabled: bool,
    pub tx_pin: Option<String>,
    pub rx_pin: Option<String>,
}

impl Serial {
    pub fn supported(port: &Port) -> bool { port.subfamily_name() == subfamily::STM32F4 }
}
