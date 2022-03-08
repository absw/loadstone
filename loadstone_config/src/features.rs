use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::{pins::PeripheralPin, port::Port};

/// Collection of Loadstone features that are optional or
/// somehow configurable.
#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub struct FeatureConfiguration {
    pub serial: Serial,
    pub boot_metrics: BootMetrics,
    pub update_signal: UpdateSignal,
    pub greetings: Greetings,
}

/// Feature that governs whether loadstone will relay boot information
/// to the application, so it can consume or display it.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum BootMetrics {
    Enabled {
        /// Support for boot timing information (time elapsed between starting
        /// Loadstone and boot).
        timing: bool
    },
    Disabled,
}

/// Custom greetings feature. If enabled, both loadstone and the associated demo app
/// will use custom greetings on startup.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Greetings {
    Default,
    Custom { loadstone: Cow<'static, str>, demo: Cow<'static, str> }
}

impl Default for Greetings {
    fn default() -> Self { Self::Default }
}

impl Default for BootMetrics {
    fn default() -> Self { Self::Disabled }
}

impl BootMetrics {
    /// Whether a given port is capable of recording boot timing information.
    pub fn timing_supported(port: &Port) -> bool {
        match port {
            Port::Stm32F412 => true,
            Port::Stm32F446 => false,
            Port::Wgm160P => false,
            Port::Max32631 => false,
        }
    }
}

/// Serial communication feature. If enabled, Loastone will provide
/// information about the boot process via serial.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Serial {
    Enabled {
        /// If enabled, loadstone will offer the option to recover a device
        /// with no bootable image via serial.
        recovery_enabled: bool,
        /// Hardware pin for serial transmission (from loadstone's perspective).
        tx_pin: PeripheralPin,
        /// Hardware pin for serial reception (from loadstone's perspective).
        rx_pin: PeripheralPin
    },
    Disabled,
}

impl Default for Serial {
    fn default() -> Self { Self::Disabled }
}

impl Serial {
    /// Whether a port is capable of supporting serial communications.
    pub fn supported(port: &Port) -> bool {
        match port {
            Port::Stm32F412 => true,
            Port::Stm32F446 => true,
            Port::Wgm160P => false,
            Port::Max32631 => false,
        }
    }

    pub fn enabled(&self) -> bool { matches!(self, Serial::Enabled { .. }) }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum UpdateSignal {
    Disabled,
    Enabled,
}

impl Default for UpdateSignal {
    fn default() -> Self { UpdateSignal::Disabled }
}
