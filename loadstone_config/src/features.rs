use serde::Serialize;

use crate::port::{family, subfamily, Port};

#[derive(Default, Clone, Serialize)]
pub struct FeatureConfiguration {
    pub serial: Serial,
    pub boot_metrics: BootMetrics,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct BootMetrics {
    pub enabled: bool,
    pub timing_enabled: bool,
}

impl BootMetrics {
    pub fn timing_supported(port: &Port) -> bool { port.family_name() == family::STM32 }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct Serial {
    pub enabled: bool,
    pub recovery_enabled: bool,
    pub tx_pin: Option<&'static str>,
    pub rx_pin: Option<&'static str>,
}

impl Serial {
    pub fn supported(port: &Port) -> bool { port.subfamily_name() == subfamily::STM32F4 }
}
