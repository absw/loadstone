//! Complex modules with business logic related to the problem
//! domain, that lay on top of abstract drivers. Devices are
//! generic, while board specifics (pins, board config) are
//! handled in the `ports` module.

pub mod boot_manager;
pub mod boot_metrics;
pub mod bootloader;
pub mod cli;
pub mod image;

/// General purpose traits that summarize requirements on devices.
pub mod traits {
    use crate::error;
    use blue_hal::hal::{flash, serial};
    use marker_blanket::marker_blanket;

    /// A supported flash must be able to read, write, and report errors
    /// to the bootloader or boot manager.
    #[marker_blanket]
    pub trait Flash: flash::ReadWrite<Error: error::Convertible> {}

    /// A supported serial must be able to read, write, read with a timeout,
    /// and report errors to the bootloader or boot manager.
    #[marker_blanket]
    pub trait Serial:
        serial::Read<Error: error::Convertible>
        + serial::Write
        + serial::TimeoutRead<Error: error::Convertible>
    {
    }
}
