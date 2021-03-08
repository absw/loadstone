//! Complex modules with business logic related to the problem
//! domain, that lay on top of abstract drivers. Devices are
//! generic, while board specifics (pins, board config) are
//! handled in the `ports` module.


pub mod bootloader;
pub mod boot_metrics;
pub mod boot_manager;
pub mod image;
pub mod cli;

/// General purpose traits that summarize requirements on devices.
pub mod traits {
    use blue_hal::hal::{flash, serial};
    use crate::error;

    /// A supported flash must be able to read, write, and report errors
    /// to the bootloader or boot manager.
    pub trait Flash: flash::ReadWrite<Error: error::Convertible> {}
    impl<T: flash::ReadWrite<Error: error::Convertible>> Flash for T {}

    /// A supported serial must be able to read, write, read with a timeout,
    /// and report errors to the bootloader or boot manager.
    pub trait Serial:
        serial::Read<Error: error::Convertible>
        + serial::Write
        + serial::TimeoutRead<Error: error::Convertible> {}
    impl<T: serial::Read<Error: error::Convertible>
        + serial::Write
        + serial::TimeoutRead<Error: error::Convertible>>
        Serial for T {}
}
