//! # Simple GPIO interface
//!
//! Separete interfaces to Input and Output pins, automatically
//! implemented by GPIOs that support such operations.
//!
//! For this project in particular, these traits are automatically implemented
//! for pins with the appropriate typestates, so there's no need for
//! manual implementation.

/// Interface to a writable pin.
pub trait OutputPin {
    fn set_low(&mut self);
    fn set_high(&mut self);
}

/// Interface to a readable pin.
pub trait InputPin {
    fn is_high(&self) -> bool;
    fn is_low(&self) -> bool;
}
