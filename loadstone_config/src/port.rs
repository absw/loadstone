use std::fmt::Display;

use crate::KB;
use enum_iterator::IntoEnumIterator;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, IntoEnumIterator)]
pub enum Port {
    Stm32F412,
    Wgm160P,
}

impl Default for Port {
    // Arbitrary default port for the purposes of seeding
    // the defaults in the web application
    fn default() -> Self { Self::Stm32F412 }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Family {
    Stm32,
    Efm32,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Subfamily {
    Stm32f4,
    Efm32Gg11,
}

impl Display for Port {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Port::Stm32F412 => "stm32f412",
            Port::Wgm160P => "wgm160p",
        })
    }
}

impl Display for Family {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Family::Stm32 => "stm32",
            Family::Efm32 => "efm32",
        })
    }
}

impl Display for Subfamily {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Subfamily::Stm32f4 => "f4",
            Subfamily::Efm32Gg11 => "gg11",
        })
    }
}

impl Port {
    pub fn family(&self) -> Family {
        match self {
            Port::Stm32F412 => Family::Stm32,
            Port::Wgm160P => Family::Efm32,
        }
    }
    pub fn subfamily(&self) -> Subfamily {
        match self {
            Port::Stm32F412 => Subfamily::Stm32f4,
            Port::Wgm160P => Subfamily::Efm32Gg11,
        }
    }

    // We might consider making these configurable later, but the need hasn't come up yet.
    pub fn linker_script_constants(&self) -> Option<LinkerScriptConstants> {
        match self {
            Port::Stm32F412 => Some(LinkerScriptConstants {
                flash: LinkerArea { origin: 0x08000000, size: KB!(80) },
                ram: LinkerArea { origin: 0x20000000, size: KB!(256) },
            }),
            Port::Wgm160P => Some(LinkerScriptConstants {
                flash: LinkerArea { origin: 0x00000000, size: KB!(1024) },
                ram: LinkerArea { origin: 0x20000000, size: KB!(128) },
            }),
            _ => None,
        }
    }
}

pub struct LinkerScriptConstants {
    pub flash: LinkerArea,
    pub ram: LinkerArea,
}

pub struct LinkerArea {
    pub origin: u32,
    pub size: usize,
}
