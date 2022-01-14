use std::fmt::Display;

use crate::KB;
use enum_iterator::IntoEnumIterator;
use serde::{Deserialize, Serialize};

/// Top level description of the hardware target. Typically a chip subfamily, but it
/// may be more or less concrete depending on the available drivers.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, IntoEnumIterator)]
pub enum Port {
    Stm32F412,
    Wgm160P,
    Maxim3263,
}

impl Default for Port {
    // Arbitrary default port for the purposes of seeding
    // the defaults in the web application
    fn default() -> Self { Self::Stm32F412 }
}

/// Supported hardware families.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Family {
    Stm32,
    Efm32,
    Maxim32,
}

/// Supported hardware subfamilies.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Subfamily {
    Stm32f4,
    Efm32Gg11,
    Maxim3263,
}

impl Display for Port {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Port::Stm32F412 => "stm32f412",
            Port::Wgm160P => "wgm160p",
            Port::Maxim3263 => "maxim3263",
        })
    }
}

impl Display for Family {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Family::Stm32 => "stm32",
            Family::Efm32 => "efm32",
            Family::Maxim32 => "maxim32",
        })
    }
}

impl Display for Subfamily {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Subfamily::Stm32f4 => "f4",
            Subfamily::Efm32Gg11 => "gg11",
            Subfamily::Maxim3263 => "63",
        })
    }
}

impl Port {
    /// Hardware family of this port.
    pub fn family(&self) -> Family {
        match self {
            Port::Stm32F412 => Family::Stm32,
            Port::Wgm160P => Family::Efm32,
            Port::Maxim3263 => Family::Maxim32,
        }
    }

    /// Hardware subfamily of this port.
    pub fn subfamily(&self) -> Subfamily {
        match self {
            Port::Stm32F412 => Subfamily::Stm32f4,
            Port::Wgm160P => Subfamily::Efm32Gg11,
            Port::Maxim3263 => Subfamily::Maxim3263,
        }
    }

    /// Constants to be propagated to the linker script for this port. This mainly
    /// defines the sections of ram and flash memory.
    // We might consider making these configurable later, but the need hasn't come up yet.
    pub fn linker_script_constants(&self) -> Option<LinkerScriptConstants> {
        match self {
            Port::Stm32F412 => Some(LinkerScriptConstants {
                flash: LinkerArea { origin: 0x08000000, size: KB!(896) },
                ram: LinkerArea { origin: 0x20000000, size: KB!(256) },
            }),
            Port::Wgm160P => Some(LinkerScriptConstants {
                flash: LinkerArea { origin: 0x00000000, size: KB!(1024) },
                ram: LinkerArea { origin: 0x20000000, size: KB!(128) },
            }),
            Port::Maxim3263 => Some(LinkerScriptConstants {
                flash: LinkerArea { origin: 0x00000000, size: KB!(2048) },
                ram: LinkerArea { origin: 0x20000000, size: KB!(512) },
            }),
        }
    }
}

/// Constants to be propagated to the linker script for this port.
pub struct LinkerScriptConstants {
    /// Available flash memory as defined in the linker script.
    pub flash: LinkerArea,
    /// Available ram memory as defined in the linker script.
    pub ram: LinkerArea,
}

/// A section of memory as defined in the linker script.
pub struct LinkerArea {
    pub origin: u32,
    pub size: usize,
}
