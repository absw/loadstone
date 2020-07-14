//! Complex modules with business logic related to the problem
//! domain, that lay on top of abstract drivers. Devices are
//! generic, while board specifics (pins, board config) are
//! handled in the `ports` module.

pub mod bootloader;
