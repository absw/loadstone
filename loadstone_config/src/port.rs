use crate::KB;
use serde::{Deserialize, Serialize};

pub mod family {
    pub static STM32: &'static str = "stm32";
    pub static EFM32: &'static str = "efm32";
}

pub mod subfamily {
    pub static STM32F4: &'static str = "stm32f4";
    pub static EFM32GG11: &'static str = "efm32gg11";
}

pub mod board {
    pub static STM32F412: &'static str = "stm32f412";
    pub static WGM160P: &'static str = "wgm160p";
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct PortLevel(pub Category, pub String, pub Vec<PortLevel>);

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Port {
    pub family: Option<PortLevel>,
    pub subfamily: Option<PortLevel>,
    pub board: Option<PortLevel>,
}

impl Port {
    pub fn family_name(&self) -> &str {
        self.family.as_ref().map(|f| f.name()).unwrap_or("Unknown")
    }
    pub fn subfamily_name(&self) -> &str {
        self.subfamily.as_ref().map(|s| s.name()).unwrap_or("Unknown")
    }
    pub fn board_name(&self) -> &str { self.board.as_ref().map(|b| b.name()).unwrap_or("Unknown") }

    // We might consider making these configurable later, but the need hasn't come up yet.
    pub fn linker_script_constants(&self) -> Option<LinkerScriptConstants> {
        match self.board_name() {
            name if name == board::STM32F412 => Some(LinkerScriptConstants {
                flash: LinkerArea { origin: 0x08000000, size: KB!(80) },
                ram: LinkerArea { origin: 0x20000000, size: KB!(256) },
            }),
            name if name == board::WGM160P => Some(LinkerScriptConstants {
                flash: LinkerArea { origin: 0x00000000, size: KB!(1024) },
                ram: LinkerArea { origin: 0x20000000, size: KB!(128) },
            }),
            _ => None,
        }
    }
}

impl PortLevel {
    pub fn category(&self) -> Category { self.0 }
    pub fn name(&self) -> &str { &self.1 }
    pub fn children(&self) -> &Vec<PortLevel> { &self.2 }
    pub fn contains(&self, descendent: Option<&PortLevel>) -> bool {
        if let Some(descendent) = descendent {
            self.children().iter().any(|c| (c == descendent) || c.contains(Some(descendent)))
        } else {
            false
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

pub mod family_names {
    pub static STM32: &'static str = "stm32";
    pub static EFM32: &'static str = "efm32";
}

pub mod subfamily_names {
    pub static STM32F4: &'static str = "stm32f4";
    pub static EFM32GG11: &'static str = "efm32gg11";
}

pub mod board_names {
    pub static STM32F412: &'static str = "stm32f412";
    pub static WGM160P: &'static str = "wgm160p";
}

pub fn port_tree() -> Vec<PortLevel> {
    vec![
        PortLevel(Category::Family, family::STM32.into(), vec![PortLevel(
            Category::Subfamily,
            subfamily::STM32F4.into(),
            vec![PortLevel(Category::Board, board::STM32F412.into(), vec![])],
        )]),
        PortLevel(Category::Family, family::EFM32.into(), vec![PortLevel(
            Category::Subfamily,
            subfamily::EFM32GG11.into(),
            vec![PortLevel(Category::Board, board::WGM160P.into(), vec![])],
        )]),
    ]
}

#[derive(PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Category {
    Family,
    Subfamily,
    Board,
}
