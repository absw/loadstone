pub mod led;
pub mod flash {
    pub mod micron_n25q128a;
}

mod bootloader_ports {
    #[cfg(feature = "stm32f412")]
    pub mod stm32f412_discovery;
}

#[cfg(feature = "stm32f412")]
pub use bootloader_ports::stm32f412_discovery::Bootloader;
