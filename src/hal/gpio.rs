//! # Simple GPIO interface
//!
//! Separate interfaces to Input and Output pins, automatically
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

/// RAII helper for output pins.
///
/// Keeps a pin high while alive.
pub struct GuardHigh<'a>
{
    pin: &'a mut dyn OutputPin,
}

/// RAII helper for output pins.
///
/// Keeps a pin low while alive.
pub struct GuardLow<'a>
{
    pin: &'a mut dyn OutputPin,
}

pub fn guard_high<'a>(pin: &'a mut dyn OutputPin) -> GuardHigh<'a> {
    pin.set_high();
    GuardHigh { pin }
}

pub fn guard_low<'a>(pin: &'a mut dyn OutputPin) -> GuardLow<'a> {
    pin.set_low();
    GuardLow { pin }
}

impl<'a> Drop for GuardHigh<'a>
{
    fn drop(&mut self) {
        self.pin.set_low();
    }
}

impl<'a> Drop for GuardLow<'a>
{
    fn drop(&mut self) {
        self.pin.set_high();
    }
}
